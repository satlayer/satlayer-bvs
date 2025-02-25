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

type StrategyManager struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewStrategyManager(chainIO io.ChainIO, contractAddr string) *StrategyManager {
	return &StrategyManager{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *StrategyManager) WithGasAdjustment(gasAdjustment float64) *StrategyManager {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *StrategyManager) WithGasPrice(gasPrice sdktypes.DecCoin) *StrategyManager {
	r.gasPrice = gasPrice
	return r
}

func (r *StrategyManager) WithGasLimit(gasLimit uint64) *StrategyManager {
	r.gasLimit = gasLimit
	return r
}

func (r *StrategyManager) AddStrategiesToWhitelist(ctx context.Context, strategies []string, thirdPartyTransfersForbiddenValues []bool) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		AddStrategiesToWhitelist: &strategymanager.AddStrategiesToWhitelist{
			Strategies:                         strategies,
			ThirdPartyTransfersForbiddenValues: thirdPartyTransfersForbiddenValues,
		},
	}
	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "AddStrategiesToWhitelist")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) RemoveStrategiesFromWhitelist(ctx context.Context, strategies []string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		RemoveStrategiesFromWhitelist: &strategymanager.RemoveStrategiesFromWhitelist{
			Strategies: strategies,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "RemoveStrategiesFromWhitelist")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) SetStrategyWhitelister(ctx context.Context, newStrategyWhitelister string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetStrategyWhitelister: &strategymanager.SetStrategyWhitelister{
			NewStrategyWhitelister: newStrategyWhitelister,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetStrategyWhitelister")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) DepositIntoStrategy(ctx context.Context, strategy string, token string, amount uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		DepositIntoStrategy: &strategymanager.DepositIntoStrategy{
			Strategy: strategy,
			Token:    token,
			Amount:   fmt.Sprintf("%d", amount),
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "DepositIntoStrategy")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) SetThirdPartyTransfersForbidden(ctx context.Context, strategy string, value bool) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetThirdPartyTransfersForbidden: &strategymanager.SetThirdPartyTransfersForbidden{
			Strategy: strategy,
			Value:    value,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetThirdPartyTransfersForbidden")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) DepositIntoStrategyWithSignature(ctx context.Context, strategy string, token string, amount uint64, staker string, publicKey cryptotypes.PubKey, stakerKeyName string) (*coretypes.ResultTx, error) {
	nodeStatus, err := r.io.QueryNodeStatus(context.Background())
	if err != nil {
		return nil, err
	}

	expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000
	chainId := r.io.GetClientCtx().ChainID

	resp, err := r.GetNonce(staker)

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
		ContractAddr: r.contractAddr,
	}

	resp, err = r.CalculateDigestHash(params)

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

	signature, err := r.io.GetSigner().SignByKeyName(bytes, stakerKeyName)

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

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "DepositIntoStrategyWithSignature")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) RemoveShares(ctx context.Context, staker string, strategy string, shares uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		RemoveShares: &strategymanager.RemoveShares{
			Staker:   staker,
			Strategy: strategy,
			Shares:   fmt.Sprintf("%d", shares),
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "RemoveShares")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) WithdrawSharesAsTokens(ctx context.Context, recipient string, strategy string, shares uint64, token string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		WithdrawSharesAsTokens: &strategymanager.WithdrawSharesAsTokens{
			Recipient: recipient,
			Strategy:  strategy,
			Shares:    fmt.Sprintf("%d", shares),
			Token:     token,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "WithdrawSharesAsTokens")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) AddShares(ctx context.Context, staker string, token string, strategy string, shares uint64) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		AddShares: &strategymanager.AddShares{
			Staker:   staker,
			Token:    token,
			Strategy: strategy,
			Shares:   fmt.Sprintf("%d", shares),
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "AddShares")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetDelegationManager: &strategymanager.SetDelegationManager{
			NewDelegationManager: newDelegationManager,
		},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetDelegationManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		Pause: &strategymanager.Pause{},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Pause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		Unpause: &strategymanager.Unpause{},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Unpause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetPauser: &strategymanager.SetPauser{NewPauser: newPauser},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetPauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetUnpauser: &strategymanager.SetUnpauser{NewUnpauser: newUnpauser},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetUnpauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) SetSlashManager(ctx context.Context, newSlashManager string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetSlashManager: &strategymanager.SetSlashManager{NewSlashManager: newSlashManager},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetSlashManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) SetStrategyFactory(ctx context.Context, newStrategyFactory string) (*coretypes.ResultTx, error) {
	msg := strategymanager.ExecuteMsg{
		SetStrategyFactory: &strategymanager.SetStrategyFactory{NewStrategyFactory: newStrategyFactory},
	}

	executeMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetStrategyFactory")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) TwoStepTransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := strategymanager.ExecuteMsg{
		TwoStepTransferOwnership: &strategymanager.TwoStepTransferOwnership{
			NewOwner: newOwner,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "TwoStepTransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) AcceptOwnership(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategymanager.ExecuteMsg{
		AcceptOwnership: &strategymanager.AcceptOwnership{},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "AcceptOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) CancelOwnershipTransfer(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := strategymanager.ExecuteMsg{
		CancelOwnershipTransfer: &strategymanager.CancelOwnershipTransfer{},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "CancelOwnershipTransfer")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *StrategyManager) GetDeposits(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetDeposits: &strategymanager.GetDeposits{
			Staker: staker,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) StakerStrategyListLength(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		StakerStrategyListLength: &strategymanager.StakerStrategyListLength{
			Staker: staker,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) GetStakerStrategyShares(staker string, strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetStakerStrategyShares: &strategymanager.GetStakerStrategyShares{
			Staker:   staker,
			Strategy: strategy,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) IsThirdPartyTransfersForbidden(strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		IsThirdPartyTransfersForbidden: &strategymanager.IsThirdPartyTransfersForbidden{
			Strategy: strategy,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) GetNonce(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetNonce: &strategymanager.GetNonce{
			Staker: staker,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) GetStakerStrategyList(staker string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetStakerStrategyList: &strategymanager.GetStakerStrategyList{
			Staker: staker,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) Owner() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		Owner: &strategymanager.Owner{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) IsStrategyWhitelisted(strategy string) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		IsStrategyWhitelisted: &strategymanager.IsStrategyWhitelisted{
			Strategy: strategy,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) CalculateDigestHash(params strategymanager.QueryDigestHashParams) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		CalculateDigestHash: &strategymanager.CalculateDigestHash{
			DigestHashParams: params,
		},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) GetStrategyWhitelister() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetStrategyWhitelister: &strategymanager.GetStrategyWhitelister{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) GetStrategyManagerState() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetStrategyManagerState: &strategymanager.GetStrategyManagerState{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) GetDepositTypeHash() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		GetDepositTypeHash: &strategymanager.GetDepositTypeHash{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) DomainTypeHash() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		DomainTypeHash: &strategymanager.DomainTypeHash{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) DomainName() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		DomainName: &strategymanager.DomainName{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) DelegationManager() (*wasmtypes.QuerySmartContractStateResponse, error) {
	msg := strategymanager.QueryMsg{
		DelegationManager: &strategymanager.DelegationManager{},
	}

	queryMsgBytes, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	return r.io.QueryContract(queryOptions)
}

func (r *StrategyManager) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  r.contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: r.gasAdjustment,
		GasPrice:      r.gasPrice,
		Gas:           r.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (r *StrategyManager) newQueryOptions(queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: r.contractAddr,
		QueryMsg:     queryMsg,
	}
}
