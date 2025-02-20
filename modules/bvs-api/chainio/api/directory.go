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
	"github.com/satlayer/satlayer-bvs/bvs-cw/directory"
)

type BvsDirectory struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewBvsDirectory(chainIO io.ChainIO, contractAddr string) *BvsDirectory {
	return &BvsDirectory{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (a *BvsDirectory) WithGasAdjustment(gasAdjustment float64) *BvsDirectory {
	a.gasAdjustment = gasAdjustment
	return a
}

func (a *BvsDirectory) WithGasPrice(gasPrice sdktypes.DecCoin) *BvsDirectory {
	a.gasPrice = gasPrice
	return a
}

func (a *BvsDirectory) WithGasLimit(gasLimit uint64) *BvsDirectory {
	a.gasLimit = gasLimit
	return a
}

func (a *BvsDirectory) RegisterBVS(ctx context.Context, bvsContract string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{RegisterBVS: &directory.RegisterBVS{
		BvsContract: bvsContract,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "RegisterBVS")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) RegisterOperator(ctx context.Context, operator string, publicKey cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	nodeStatus, err := a.io.QueryNodeStatus(context.Background())
	if err != nil {
		return nil, err
	}
	expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000

	randomStr, err := utils.GenerateRandomString(16)
	if err != nil {
		return nil, err
	}
	salt := "salt" + randomStr

	msgHashResp, err := a.CalculateDigestHash(publicKey, operator, salt, expiry)
	if err != nil {
		return nil, err
	}
	bytes, err := base64.StdEncoding.DecodeString(msgHashResp.DigestHash)
	if err != nil {
		return nil, err
	}
	sig, err := a.io.GetSigner().Sign(bytes)
	if err != nil {
		return nil, err
	}
	executeMsg := directory.ExecuteMsg{RegisterOperatorToBVS: &directory.RegisterOperatorToBVS{
		Operator:     operator,
		PublicKey:    base64.StdEncoding.EncodeToString(publicKey.Bytes()),
		ContractAddr: a.contractAddr,
		SignatureWithSaltAndExpiry: directory.ExecuteSignatureWithSaltAndExpiry{
			Signature: sig,
			Salt:      base64.StdEncoding.EncodeToString([]byte(salt)),
			Expiry:    expiry,
		},
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "RegisterOperator")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) DeregisterOperator(ctx context.Context, operator string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{DeregisterOperatorFromBVS: &directory.DeregisterOperatorFromBVS{Operator: operator}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "DeregisterOperator")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) UpdateMetadataURI(ctx context.Context, metadataURI string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{UpdateBVSMetadataURI: &directory.UpdateBVSMetadataURI{MetadataURI: metadataURI}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "UpdateMetadataURI")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) CancelSalt(ctx context.Context, salt string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{CancelSalt: &directory.CancelSalt{Salt: base64.StdEncoding.EncodeToString([]byte(salt))}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "CancelSalt")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{TransferOwnership: &directory.TransferOwnership{NewOwner: newOwner}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "TransferOwnership")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{Pause: &directory.Pause{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "Pause")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{Unpause: &directory.Unpause{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "Unpause")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetPauser: &directory.SetPauser{NewPauser: newPauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "SetPauser")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetUnpauser: &directory.SetUnpauser{NewUnpauser: newUnpauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "SetUnpauser")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) SetDelegationManager(ctx context.Context, delegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetDelegationManager: &directory.SetDelegationManager{DelegationManager: delegationManager}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := a.newExecuteOptions(a.contractAddr, executeMsgBytes, "SetDelegationManager")

	return a.io.SendTransaction(ctx, executeOptions)
}

func (a *BvsDirectory) QueryOperator(bvs, operator string) (*directory.OperatorStatusResponse, error) {
	result := new(directory.OperatorStatusResponse)
	queryMsg := directory.QueryMsg{GetOperatorStatus: &directory.GetOperatorStatus{
		Operator: operator,
		Bvs:      bvs,
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

func (a *BvsDirectory) CalculateDigestHash(operatorPublicKey cryptotypes.PubKey, bvs, salt string, expiry int64) (*directory.DigestHashResponse, error) {
	result := new(directory.DigestHashResponse)
	queryMsg := &directory.QueryMsg{CalculateDigestHash: &directory.CalculateDigestHash{
		OperatorPublicKey: base64.StdEncoding.EncodeToString(operatorPublicKey.Bytes()),
		Bvs:               bvs,
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

func (a *BvsDirectory) IsSaltSpent(operator, salt string) (*directory.SaltResponse, error) {
	result := new(directory.SaltResponse)
	queryMsg := directory.QueryMsg{IsSaltSpent: &directory.IsSaltSpent{
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

func (a *BvsDirectory) GetDelegationManager() (*directory.DelegationResponse, error) {
	result := new(directory.DelegationResponse)
	queryMsg := directory.QueryMsg{
		GetDelegationManager: &directory.GetDelegationManager{},
	}
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

func (a *BvsDirectory) GetOwner() (*directory.OwnerResponse, error) {
	result := new(directory.OwnerResponse)
	queryMsg := directory.QueryMsg{
		GetOwner: &directory.GetOwner{},
	}
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

func (a *BvsDirectory) GetOperatorBVSRegistrationTypeHash() (*directory.RegistrationTypeHashResponse, error) {
	result := new(directory.RegistrationTypeHashResponse)
	queryMsg := directory.QueryMsg{
		GetOperatorBVSRegistrationTypeHash: &directory.GetOperatorBVSRegistrationTypeHash{},
	}
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

func (a *BvsDirectory) GetDomainTypeHash() (*directory.DomainTypeHashResponse, error) {
	result := new(directory.DomainTypeHashResponse)
	queryMsg := directory.QueryMsg{
		GetDomainTypeHash: &directory.GetDomainTypeHash{},
	}
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

func (a *BvsDirectory) GetDomainName() (*directory.DomainNameResponse, error) {
	result := new(directory.DomainNameResponse)
	queryMsg := directory.QueryMsg{
		GetDomainName: &directory.GetDomainName{},
	}
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

func (a *BvsDirectory) GetBVSInfo(bvsHash string) (*directory.BVSInfoResponse, error) {
	result := new(directory.BVSInfoResponse)
	queryMsg := directory.QueryMsg{GetBVSInfo: &directory.GetBVSInfo{BvsHash: bvsHash}}
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

func (a *BvsDirectory) newExecuteOptions(contractAddr string, executeMsg []byte, memo string) types.ExecuteOptions {
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

func (a *BvsDirectory) newQueryOptions(contractAddr string, queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: contractAddr,
		QueryMsg:     queryMsg,
	}
}
