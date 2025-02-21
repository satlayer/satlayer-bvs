package api

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	strategymanager "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-manager"
)

type StrategyManager interface {
	WithGasAdjustment(gasAdjustment float64) StrategyManager
	WithGasPrice(gasPrice sdktypes.DecCoin) StrategyManager
	WithGasLimit(gasLimit uint64) StrategyManager

	BindClient(string)

	AddStrategiesToWhitelist(ctx context.Context, strategies []string, thirdPartyTransfersForbiddenValues []bool) (*coretypes.ResultTx, error)
	RemoveStrategiesFromWhitelist(ctx context.Context, strategies []string) (*coretypes.ResultTx, error)
	SetStrategyWhitelister(ctx context.Context, newStrategyWhitelister string) (*coretypes.ResultTx, error)
	DepositIntoStrategy(ctx context.Context, strategy string, token string, amount uint64) (*coretypes.ResultTx, error)
	SetThirdPartyTransfersForbidden(ctx context.Context, strategy string, value bool) (*coretypes.ResultTx, error)
	DepositIntoStrategyWithSignature(ctx context.Context, strategy string, token string, amount uint64, staker string, publicKey cryptotypes.PubKey, stakerKeyName string) (*coretypes.ResultTx, error)
	RemoveShares(ctx context.Context, staker string, strategy string, shares uint64) (*coretypes.ResultTx, error)
	WithdrawSharesAsTokens(ctx context.Context, recipient string, strategy string, shares uint64, token string) (*coretypes.ResultTx, error)
	AddShares(ctx context.Context, staker string, token string, strategy string, shares uint64) (*coretypes.ResultTx, error)
	SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)
	SetSlashManager(ctx context.Context, newSlashManager string) (*coretypes.ResultTx, error)
	SetStrategyFactory(ctx context.Context, newStrategyFactory string) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)

	GetDeposits(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error)
	StakerStrategyListLength(staker string) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetStakerStrategyShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error)
	IsThirdPartyTransfersForbidden(strategy string) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetNonce(staker string) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetStakerStrategyList(staker string) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetOwner() (*wasmtypes.QuerySmartContractStateResponse, error)
	IsStrategyWhitelisted(strategy string) (*wasmtypes.QuerySmartContractStateResponse, error)
	CalculateDigestHash(params strategymanager.QueryDigestHashParams) (*wasmtypes.QuerySmartContractStateResponse, error)
	GetStrategyWhitelister() (*wasmtypes.QuerySmartContractStateResponse, error)
	GetStrategyManagerState() (*wasmtypes.QuerySmartContractStateResponse, error)
	GetDepositTypehash() (*wasmtypes.QuerySmartContractStateResponse, error)
	GetDomainTypehash() (*wasmtypes.QuerySmartContractStateResponse, error)
	GetDomainName() (*wasmtypes.QuerySmartContractStateResponse, error)
	GetDelegationManager() (*wasmtypes.QuerySmartContractStateResponse, error)
}

type strategyManagerImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func (a *strategyManagerImpl) WithGasAdjustment(gasAdjustment float64) StrategyManager {
	a.gasAdjustment = gasAdjustment
	return a
}

func (a *strategyManagerImpl) WithGasPrice(gasPrice sdktypes.DecCoin) StrategyManager {
	a.gasPrice = gasPrice
	return a
}

func (a *strategyManagerImpl) WithGasLimit(gasLimit uint64) StrategyManager {
	a.gasLimit = gasLimit
	return a
}

func (a *strategyManagerImpl) BindClient(contractAddress string) {
	a.executeOptions = &types.ExecuteOptions{
		ContractAddr:  contractAddress,
		ExecuteMsg:    []byte{},
		Funds:         "",
		GasAdjustment: a.gasAdjustment,
		GasPrice:      a.gasPrice,
		Gas:           a.gasLimit,
		Memo:          "test tx",
		Simulate:      true,
	}

	a.queryOptions = &types.QueryOptions{
		ContractAddr: contractAddress,
		QueryMsg:     []byte{},
	}
}

func (a *strategyManagerImpl) AddStrategiesToWhitelist(ctx context.Context, strategies []string, thirdPartyTransfersForbiddenValues []bool) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		AddStrategiesToWhitelist: &strategymanager.AddStrategiesToWhitelist{
			Strategies:                         strategies,
			ThirdPartyTransfersForbiddenValues: thirdPartyTransfersForbiddenValues,
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) RemoveStrategiesFromWhitelist(ctx context.Context, strategies []string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		RemoveStrategiesFromWhitelist: &strategymanager.RemoveStrategiesFromWhitelist{
			Strategies: strategies,
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) SetStrategyWhitelister(ctx context.Context, newStrategyWhitelister string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetStrategyWhitelister: &strategymanager.SetStrategyWhitelister{
			NewStrategyWhitelister: newStrategyWhitelister,
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) DepositIntoStrategy(ctx context.Context, strategy string, token string, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		DepositIntoStrategy: &strategymanager.DepositIntoStrategy{
			Strategy: strategy,
			Token:    token,
			Amount:   fmt.Sprintf("%d", amount),
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) SetThirdPartyTransfersForbidden(ctx context.Context, strategy string, value bool) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetThirdPartyTransfersForbidden: &strategymanager.SetThirdPartyTransfersForbidden{
			Strategy: strategy,
			Value:    value,
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) DepositIntoStrategyWithSignature(ctx context.Context, strategy string, token string, amount uint64, staker string, publicKey cryptotypes.PubKey, stakerKeyName string) (*coretypes.ResultTx, error) {
	nodeStatus, err := a.io.QueryNodeStatus(context.Background())
	if err != nil {
		return nil, err
	}

	expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000
	chainId := a.io.GetClientCtx().ChainID
	contracAddr := a.executeOptions.ContractAddr

	resp, err := a.GetNonce(staker)

	if err != nil {
		return nil, err
	}

	var nonceRes strategymanager.NonceResponse
	err = json.Unmarshal(resp.Data, &nonceRes)

	if err != nil {
		return nil, err
	}

	params := strategymanager.QueryDigestHashParams{
		Staker:       staker,
		PublicKey:    base64.StdEncoding.EncodeToString(publicKey.Bytes()),
		Strategy:     strategy,
		Token:        token,
		Amount:       fmt.Sprintf("%d", amount),
		Nonce:        nonceRes.Nonce,
		Expiry:       expiry,
		ChainID:      chainId,
		ContractAddr: contracAddr,
	}

	resp, err = a.CalculateDigestHash(params)

	if err != nil {
		return nil, err
	}

	var hashRes strategymanager.CalculateDigestHashResponse
	err = json.Unmarshal(resp.Data, &hashRes)

	if err != nil {
		return nil, err
	}

	bytes, err := base64.StdEncoding.DecodeString(hashRes.DigestHash)
	if err != nil {
		return nil, err
	}

	signature, err := a.io.GetSigner().SignByKeyName(bytes, stakerKeyName)

	if err != nil {
		return nil, err
	}

	msg := strategymanager.ExecuteMsg{
		DepositIntoStrategyWithSignature: &strategymanager.DepositIntoStrategyWithSignature{
			Strategy:  strategy,
			Token:     token,
			Amount:    fmt.Sprintf("%d", amount),
			Staker:    staker,
			PublicKey: base64.StdEncoding.EncodeToString(publicKey.Bytes()),
			Expiry:    expiry,
			Signature: signature,
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) RemoveShares(ctx context.Context, staker string, strategy string, shares uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		RemoveShares: &strategymanager.RemoveShares{
			Staker:   staker,
			Strategy: strategy,
			Shares:   fmt.Sprintf("%d", shares),
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) WithdrawSharesAsTokens(ctx context.Context, recipient string, strategy string, shares uint64, token string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		WithdrawSharesAsTokens: &strategymanager.WithdrawSharesAsTokens{
			Recipient: recipient,
			Strategy:  strategy,
			Shares:    fmt.Sprintf("%d", shares),
			Token:     token,
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) AddShares(ctx context.Context, staker string, token string, strategy string, shares uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		AddShares: &strategymanager.AddShares{
			Staker:   staker,
			Token:    token,
			Strategy: strategy,
			Shares:   fmt.Sprintf("%d", shares),
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetDelegationManager: &strategymanager.SetDelegationManager{
			NewDelegationManager: newDelegationManager,
		},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		Pause: &strategymanager.Pause{},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		Unpause: &strategymanager.Unpause{},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetPauser: &strategymanager.SetPauser{NewPauser: newPauser},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetUnpauser: &strategymanager.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) SetSlashManager(ctx context.Context, newSlashManager string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetSlashManager: &strategymanager.SetSlashManager{NewSlashManager: newSlashManager},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) SetStrategyFactory(ctx context.Context, newStrategyFactory string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetStrategyFactory: &strategymanager.SetStrategyFactory{NewStrategyFactory: newStrategyFactory},
	}

	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		TransferOwnership: &strategymanager.TransferOwnership{NewOwner: newOwner},
	}
	return a.execute(ctx, msg)
}

func (a *strategyManagerImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.executeOptions).ExecuteMsg = msgBytes
	return a.io.SendTransaction(ctx, *a.executeOptions)
}

func (a *strategyManagerImpl) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*a.queryOptions).QueryMsg = msgBytes
	return a.io.QueryContract(*a.queryOptions)
}

func (a *strategyManagerImpl) GetDeposits(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetDeposits: &strategymanager.GetDeposits{
			Staker: staker,
			// TODO: what happen to strategy field is not present on the Rust side
			// Strategy: strategy,
		},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) StakerStrategyListLength(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		StakerStrategyListLength: &strategymanager.StakerStrategyListLength{
			Staker: staker,
		},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetStakerStrategyShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetStakerStrategyShares: &strategymanager.GetStakerStrategyShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) IsThirdPartyTransfersForbidden(strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		IsThirdPartyTransfersForbidden: &strategymanager.IsThirdPartyTransfersForbidden{
			Strategy: strategy,
		},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetNonce(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetNonce: &strategymanager.GetNonce{
			Staker: staker,
		},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetStakerStrategyList(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetStakerStrategyList: &strategymanager.GetStakerStrategyList{
			Staker: staker,
		},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetOwner() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetOwner: &strategymanager.GetOwner{},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) IsStrategyWhitelisted(strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		IsStrategyWhitelisted: &strategymanager.IsStrategyWhitelisted{
			Strategy: strategy,
		},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) CalculateDigestHash(params strategymanager.QueryDigestHashParams) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		CalculateDigestHash: &strategymanager.CalculateDigestHash{
			DigstHashParams: params,
		},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetStrategyWhitelister() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetStrategyWhitelister: &strategymanager.GetStrategyWhitelister{},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetStrategyManagerState() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetStrategyManagerState: &strategymanager.GetStrategyManagerState{},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetDepositTypehash() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetDepositTypehash: &strategymanager.GetDepositTypehash{},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetDomainTypehash() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetDomainTypehash: &strategymanager.GetDomainTypehash{},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetDomainName() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetDomainName: &strategymanager.GetDomainName{},
	}

	return a.query(msg)
}

func (a *strategyManagerImpl) GetDelegationManager() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetDelegationManager: &strategymanager.GetDelegationManager{},
	}

	return a.query(msg)
}

func NewStrategyManager(chainIO io.ChainIO) StrategyManager {
	return &strategyManagerImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}
