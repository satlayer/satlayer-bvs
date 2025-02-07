package io

import (
	"context"
	"fmt"
	"math/big"
	"reflect"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/common"
	sdktypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"github.com/satlayer/satlayer-bvs/bvs-api/signer"
)

type ETHChainIO interface {
	SendTransaction(ctx context.Context, params types.ETHExecuteOptions) (*sdktypes.Receipt, error)
	ExecuteContract(ctx context.Context, params types.ETHExecuteOptions) (*sdktypes.Transaction, error)
	CallContract(ctx context.Context, params types.ETHCallOptions, result interface{}) error
	GetLatestBlockNumber(ctx context.Context) (uint64, error)
	GetChainID(ctx context.Context) (*big.Int, error)
	CreateAccount(pwd string) (accounts.Account, error)
	ImportKey(privateKeyHex string, pwd string) (accounts.Account, error)
	ListAccounts() []accounts.Account
	LockAccount(address common.Address) error
	SignHash(wallet types.ETHWallet, hash []byte) ([]byte, error)
	GetETHClient() *ethclient.Client
	Close()
}

type ethChainIO struct {
	client            *ethclient.Client
	signer            *signer.ETHSigner
	logger            logger.Logger
	metricsIndicators transactionprocess.Indicators
	params            types.TxManagerParams
	ks                *keystore.KeyStore
	chainID           *big.Int
}

func NewETHChainIO(endpoint string, keystorePath string, logger logger.Logger, metricsIndicators transactionprocess.Indicators, params types.TxManagerParams) (ETHChainIO, error) {
	client, err := ethclient.Dial(endpoint)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to ethereum node: %w", err)
	}
	ks := keystore.NewKeyStore(keystorePath, keystore.LightScryptN, keystore.LightScryptP)
	chainID, err := client.ChainID(context.Background())
	if err != nil {
		return nil, fmt.Errorf("failed to retrieve chain ID: %w", err)
	}
	return &ethChainIO{
		client:            client,
		signer:            signer.NewETHSigner(client, chainID),
		logger:            logger,
		metricsIndicators: metricsIndicators,
		params:            params,
		ks:                ks,
		chainID:           chainID,
	}, nil
}

func (e *ethChainIO) SendTransaction(ctx context.Context, params types.ETHExecuteOptions) (*sdktypes.Receipt, error) {
	e.metricsIndicators.IncrementProcessingTxCount()
	defer e.metricsIndicators.DecrementProcessingTxCount()

	startTime := time.Now()
	speedups := 0
	var (
		txResp *sdktypes.Transaction
		err    error
	)

	for attempt := 0; attempt < e.params.MaxRetries; attempt++ {
		txResp, err = e.ExecuteContract(ctx, params)
		if err == nil {
			break
		}
		e.logger.Warn("Failed to send transaction", logger.WithField("attempt", attempt+1), logger.WithField("err", err))
		if attempt == e.params.MaxRetries-1 {
			e.metricsIndicators.IncrementProcessedTxsTotal("failure")
			return nil, fmt.Errorf("max retries exceeded: %w", err)
		}
		time.Sleep(e.params.RetryInterval)
		speedups++
	}

	if txResp == nil {
		e.metricsIndicators.IncrementProcessedTxsTotal("failure")
		return nil, fmt.Errorf("failed to send transaction after %d attempts", e.params.MaxRetries)
	}

	e.metricsIndicators.ObserveBroadcastLatencyMs(time.Since(startTime).Milliseconds())

	confirmedTxResp, err := e.waitForConfirmation(ctx, txResp.Hash())
	if err != nil {
		e.metricsIndicators.IncrementProcessedTxsTotal("failure")
		return nil, err
	}

	e.metricsIndicators.ObserveConfirmationLatencyMs(time.Since(startTime).Milliseconds())
	e.metricsIndicators.ObserveSpeedups(speedups)
	e.metricsIndicators.IncrementProcessedTxsTotal("success")
	return confirmedTxResp, nil
}

func (e *ethChainIO) waitForConfirmation(ctx context.Context, hash common.Hash) (*sdktypes.Receipt, error) {
	queryTicker := time.NewTicker(time.Second)
	defer queryTicker.Stop()

	timeout := time.After(e.params.ConfirmationTimeout)

	for {
		receipt, err := e.client.TransactionReceipt(ctx, hash)
		if err == nil {
			return receipt, nil
		}

		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-timeout:
			return nil, fmt.Errorf("transaction confirmation timed out")
		case <-queryTicker.C:
			continue
		}
	}
}

func (e *ethChainIO) ExecuteContract(ctx context.Context, params types.ETHExecuteOptions) (*sdktypes.Transaction, error) {
	if params.ContractAddr == (common.Address{}) {
		return nil, fmt.Errorf("contract address cannot be zero address")
	}

	if _, exists := params.ContractABI.Methods[params.Method]; !exists {
		return nil, fmt.Errorf("method %s not found in ABI", params.Method)
	}

	input, err := params.ContractABI.Pack(params.Method, params.Args...)
	if err != nil {
		return nil, fmt.Errorf("failed to pack input: %w", err)
	}

	auth, err := e.getTransactor(params.FromAddr, params.PWD)
	if err != nil {
		return nil, err
	}
	signedTx, err := e.signer.BuildAndSignTx(ctx, auth.Signer, params.FromAddr, params.ContractAddr, e.params.ETHGasFeeCapAdjustmentRate, e.params.ETHGasLimitAdjustmentRate, e.params.GasLimit, input)
	if err != nil {
		return nil, err
	}
	err = e.client.SendTransaction(ctx, signedTx)
	if err != nil {
		return nil, fmt.Errorf("failed to send transaction: %w", err)
	}
	return signedTx, nil
}

func (e *ethChainIO) CallContract(ctx context.Context, params types.ETHCallOptions, result interface{}) error {
	input, err := params.ContractABI.Pack(params.Method, params.Args...)
	if err != nil {
		return fmt.Errorf("failed to pack input: %w", err)
	}
	msg := ethereum.CallMsg{
		To:   &params.ContractAddr,
		Data: input,
	}

	output, err := e.client.CallContract(ctx, msg, nil)
	if err != nil {
		return fmt.Errorf("failed to call contract: %w", err)
	}

	rv := reflect.ValueOf(result)
	if rv.Kind() != reflect.Ptr || rv.IsNil() {
		return fmt.Errorf("result must be a non-nil pointer")
	}

	return params.ContractABI.UnpackIntoInterface(result, params.Method, output)
}

func (e *ethChainIO) GetLatestBlockNumber(ctx context.Context) (uint64, error) {
	return e.client.BlockNumber(ctx)
}

func (e *ethChainIO) GetChainID(ctx context.Context) (*big.Int, error) {
	return e.client.ChainID(ctx)
}

func (e *ethChainIO) CreateAccount(pwd string) (accounts.Account, error) {
	return e.ks.NewAccount(pwd)
}

func (e *ethChainIO) ImportKey(privateKeyHex string, pwd string) (accounts.Account, error) {
	key, err := crypto.HexToECDSA(privateKeyHex)
	if err != nil {
		return accounts.Account{}, err
	}

	return e.ks.ImportECDSA(key, pwd)
}

func (e *ethChainIO) ListAccounts() []accounts.Account {
	return e.ks.Accounts()
}

func (e *ethChainIO) LockAccount(address common.Address) error {
	return e.ks.Lock(address)
}

func (e *ethChainIO) getTransactor(address common.Address, pwd string) (*bind.TransactOpts, error) {
	account := accounts.Account{Address: address}

	if _, err := e.ks.Find(account); err != nil {
		return nil, fmt.Errorf("account not found: %w", err)
	}

	if err := e.ks.Unlock(account, pwd); err != nil {
		return nil, fmt.Errorf("failed to unlock account: %w", err)
	}

	return bind.NewKeyStoreTransactorWithChainID(e.ks, account, e.chainID)
}

func (e *ethChainIO) SignHash(wallet types.ETHWallet, hash []byte) ([]byte, error) {
	account := accounts.Account{Address: wallet.FromAddr}

	if _, err := e.ks.Find(account); err != nil {
		return nil, fmt.Errorf("account not found: %w", err)
	}
	return e.ks.SignHashWithPassphrase(account, wallet.PWD, hash)
}

func (e *ethChainIO) GetETHClient() *ethclient.Client {
	return e.client
}

func (e *ethChainIO) Close() {
	e.client.Close()
	e.signer = nil
	e.ks = nil
}
