package api

import (
	"context"
	"encoding/base64"
	"encoding/json"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-api/utils"
)

type BVSDirectory interface {
	WithGasAdjustment(gasAdjustment float64) BVSDirectory
	WithGasPrice(gasPrice sdktypes.DecCoin) BVSDirectory
	WithGasLimit(gasLimit uint64) BVSDirectory

	RegisterBVS(ctx context.Context, bvsContract string) (*coretypes.ResultTx, error)
	RegisterOperator(ctx context.Context, operator string, publicKey cryptotypes.PubKey) (*coretypes.ResultTx, error)
	DeregisterOperator(ctx context.Context, operator string) (*coretypes.ResultTx, error)
	UpdateMetadataURI(ctx context.Context, metadataURI string) (*coretypes.ResultTx, error)
	CancelSalt(ctx context.Context, salt string) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)
	SetDelegationManager(ctx context.Context, delegationManager string) (*coretypes.ResultTx, error)

	QueryOperator(bvs, operator string) (*types.QueryOperatorResp, error)
	CalculateDigestHash(operatorPublicKey cryptotypes.PubKey, bvs, salt string, expiry uint64) (*types.CalculateDigestHashResp, error)
	IsSaltSpent(operator, salt string) (*types.IsSaltSpentResp, error)
	GetDelegationManager() (*types.GetDelegationManagerResp, error)
	GetOwner() (*types.GetOwnerResp, error)
	GetOperatorBVSRegistrationTypeHash() (*types.GetOperatorBVSRegistrationTypeHashResp, error)
	GetDomainTypeHash() (*types.GetDomainTypeHashResp, error)
	GetDomainName() (*types.GetDomainNameResp, error)
	GetBVSInfo(bvsHash string) (*types.GetBVSInfoResp, error)
}

type bvsDirectoryImpl struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func (a *bvsDirectoryImpl) WithGasAdjustment(gasAdjustment float64) BVSDirectory {
	a.gasAdjustment = gasAdjustment
	return a
}

func (a *bvsDirectoryImpl) WithGasPrice(gasPrice sdktypes.DecCoin) BVSDirectory {
	a.gasPrice = gasPrice
	return a
}

func (a *bvsDirectoryImpl) WithGasLimit(gasLimit uint64) BVSDirectory {
	a.gasLimit = gasLimit
	return a
}

func (a *bvsDirectoryImpl) RegisterBVS(ctx context.Context, bvsContract string) (*coretypes.ResultTx, error) {
	executeMsg := types.RegisterBVSReq{RegisterBVS: types.RegisterBVS{
		BVSContract: bvsContract,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "RegisterBVS")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) RegisterOperator(ctx context.Context, operator string, publicKey cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	nodeStatus, err := a.io.QueryNodeStatus(context.Background())
	if err != nil {
		return nil, err
	}
	expiry := uint64(nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000)

	randomStr, err := utils.GenerateRandomString(16)
	if err != nil {
		return nil, err
	}
	salt := "salt" + randomStr

	msgHashResp, err := a.CalculateDigestHash(publicKey, operator, salt, expiry)
	if err != nil {
		return nil, err
	}
	sig, err := a.io.GetSigner().Sign(msgHashResp.DigestHash)
	if err != nil {
		return nil, err
	}
	executeMsg := types.RegisterOperatorReq{RegisterOperator: types.RegisterOperator{
		Operator:     operator,
		PublicKey:    base64.StdEncoding.EncodeToString(publicKey.Bytes()),
		ContractAddr: a.contractAddr,
		SignatureWithSaltAndExpiry: types.SignatureWithSaltAndExpiry{
			Sig:    sig,
			Salt:   base64.StdEncoding.EncodeToString([]byte(salt)),
			Expiry: expiry,
		},
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "RegisterOperator")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) DeregisterOperator(ctx context.Context, operator string) (*coretypes.ResultTx, error) {
	executeMsg := types.DeregisterOperatorReq{DeregisterOperator: types.DeregisterOperator{Operator: operator}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "DeregisterOperator")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) UpdateMetadataURI(ctx context.Context, metadataURI string) (*coretypes.ResultTx, error) {
	executeMsg := types.UpdateMetadataURIReq{UpdateMetadataURI: types.UpdateMetadataURI{MetadataURI: metadataURI}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "UpdateMetadataURI")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) CancelSalt(ctx context.Context, salt string) (*coretypes.ResultTx, error) {
	executeMsg := types.CancelSaltReq{CancelSalt: types.CancelSalt{Salt: base64.StdEncoding.EncodeToString([]byte(salt))}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "CancelSalt")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := types.DirectoryTransferOwnershipReq{TransferOwnership: types.DirectoryTransferOwnership{NewOwner: newOwner}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "TransferOwnership")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := types.BVSDirectoryPauseReq{}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "Pause")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := types.BVSDirectoryUnpauseReq{}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "Unpause")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := types.BVSDirectorySetPauserReq{SetPauser: types.BVSDirectorySetPauser{NewPauser: newPauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "SetPauser")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := types.BVSDirectorySetUnpauserReq{SetUnpauser: types.BVSDirectorySetUnpauser{NewUnpauser: newUnpauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "SetUnpauser")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) SetDelegationManager(ctx context.Context, delegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := types.BVSDirectorySetDelegationManagerReq{SetDelegationManager: types.BVSDirectorySetDelegationManager{DelegationManager: delegationManager}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "SetDelegationManager")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *bvsDirectoryImpl) QueryOperator(bvs, operator string) (*types.QueryOperatorResp, error) {
	result := new(types.QueryOperatorResp)
	queryMsg := types.GetOperatorStatusReq{GetOperatorStatus: types.GetOperatorStatus{
		Operator: operator,
		BVS:      bvs,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) CalculateDigestHash(operatorPublicKey cryptotypes.PubKey, bvs, salt string, expiry uint64) (*types.CalculateDigestHashResp, error) {
	result := new(types.CalculateDigestHashResp)
	queryMsg := types.CalculateDigestHashReq{CalculateDigestHash: types.CalculateDigestHash{
		OperatorPublicKey: base64.StdEncoding.EncodeToString(operatorPublicKey.Bytes()),
		BVS:               bvs,
		Salt:              base64.StdEncoding.EncodeToString([]byte(salt)),
		Expiry:            expiry,
		ContractAddr:      a.contractAddr,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) IsSaltSpent(operator, salt string) (*types.IsSaltSpentResp, error) {
	result := new(types.IsSaltSpentResp)
	queryMsg := types.IsSaltSpentReq{IsSaltSpent: types.IsSaltSpent{
		Operator: operator,
		Salt:     salt,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) GetDelegationManager() (*types.GetDelegationManagerResp, error) {
	result := new(types.GetDelegationManagerResp)
	queryMsg := types.GetDelegationManagerReq{}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) GetOwner() (*types.GetOwnerResp, error) {
	result := new(types.GetOwnerResp)
	queryMsg := types.GetOwnerReq{}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) GetOperatorBVSRegistrationTypeHash() (*types.GetOperatorBVSRegistrationTypeHashResp, error) {
	result := new(types.GetOperatorBVSRegistrationTypeHashResp)
	queryMsg := types.GetOperatorBVSRegistrationTypeHashReq{}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) GetDomainTypeHash() (*types.GetDomainTypeHashResp, error) {
	result := new(types.GetDomainTypeHashResp)
	queryMsg := types.GetDomainTypeHashReq{}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) GetDomainName() (*types.GetDomainNameResp, error) {
	result := new(types.GetDomainNameResp)
	queryMsg := types.GetDomainNameReq{}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) GetBVSInfo(bvsHash string) (*types.GetBVSInfoResp, error) {
	result := new(types.GetBVSInfoResp)
	queryMsg := types.GetBVSInfoReq{GetBVSInfo: types.GetBVSInfo{BVSHash: bvsHash}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := a.newQueryOptions(a.contractAddr, queryMsgBytes)
	resp, err := a.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (a *bvsDirectoryImpl) newExecuteOptions(contractAddr string, executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: a.gasAdjustment,
		GasPrice:      a.gasPrice,
		Gas:           a.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (a *bvsDirectoryImpl) newQueryOptions(contractAddr string, queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: contractAddr,
		QueryMsg:     queryMsg,
	}
}

func NewBVSDirectoryImpl(chainIO io.ChainIO, contractAddr string) BVSDirectory {
	return &bvsDirectoryImpl{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}
