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

type Directory struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewDirectory(chainIO io.ChainIO, contractAddr string) *Directory {
	return &Directory{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      700000,
	}
}

func (r *Directory) WithGasAdjustment(gasAdjustment float64) *Directory {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *Directory) WithGasPrice(gasPrice sdktypes.DecCoin) *Directory {
	r.gasPrice = gasPrice
	return r
}

func (r *Directory) WithGasLimit(gasLimit uint64) *Directory {
	r.gasLimit = gasLimit
	return r
}

func (r *Directory) RegisterBvs(ctx context.Context, bvsContract string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{RegisterBvs: &directory.RegisterBvs{
		BvsContract: bvsContract,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "RegisterBvs")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) RegisterOperator(ctx context.Context, operator string, publicKey cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	nodeStatus, err := r.io.QueryNodeStatus(context.Background())
	if err != nil {
		return nil, err
	}
	expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000

	randomStr, err := utils.GenerateRandomString(16)
	if err != nil {
		return nil, err
	}
	salt := "salt" + randomStr

	msgHashResp, err := r.CalculateDigestHash(publicKey, operator, salt, expiry)
	if err != nil {
		return nil, err
	}
	bytes, err := base64.StdEncoding.DecodeString(msgHashResp.DigestHash)
	if err != nil {
		return nil, err
	}
	sig, err := r.io.GetSigner().Sign(bytes)
	if err != nil {
		return nil, err
	}
	executeMsg := directory.ExecuteMsg{RegisterOperatorToBvs: &directory.RegisterOperatorToBvs{
		Operator:     operator,
		PublicKey:    base64.StdEncoding.EncodeToString(publicKey.Bytes()),
		ContractAddr: r.contractAddr,
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
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "RegisterOperator")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) DeregisterOperator(ctx context.Context, operator string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{DeregisterOperatorFromBvs: &directory.DeregisterOperatorFromBvs{Operator: operator}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "DeregisterOperator")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) UpdateMetadataURI(ctx context.Context, metadataURI string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{UpdateBvsMetadataURI: &directory.UpdateBvsMetadataURI{MetadataURI: metadataURI}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "UpdateMetadataURI")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) CancelSalt(ctx context.Context, salt string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{CancelSalt: &directory.CancelSalt{Salt: base64.StdEncoding.EncodeToString([]byte(salt))}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "CancelSalt")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{TransferOwnership: &directory.TransferOwnership{NewOwner: newOwner}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "TransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{Pause: &directory.Pause{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "Pause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{Unpause: &directory.Unpause{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "Unpause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetPauser: &directory.SetPauser{NewPauser: newPauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "SetPauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetUnpauser: &directory.SetUnpauser{NewUnpauser: newUnpauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "SetUnpauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) SetDelegationManager(ctx context.Context, delegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetDelegationManager: &directory.SetDelegationManager{DelegationManager: delegationManager}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.contractAddr, executeMsgBytes, "SetDelegationManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) QueryOperator(bvs, operator string) (*directory.OperatorStatusResponse, error) {
	result := new(directory.OperatorStatusResponse)
	queryMsg := directory.QueryMsg{GetOperatorStatus: &directory.GetOperatorStatus{
		Operator: operator,
		Bvs:      bvs,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) CalculateDigestHash(operatorPublicKey cryptotypes.PubKey, bvs, salt string, expiry int64) (*directory.DigestHashResponse, error) {
	result := new(directory.DigestHashResponse)
	queryMsg := &directory.QueryMsg{CalculateDigestHash: &directory.CalculateDigestHash{
		OperatorPublicKey: base64.StdEncoding.EncodeToString(operatorPublicKey.Bytes()),
		Bvs:               bvs,
		Salt:              base64.StdEncoding.EncodeToString([]byte(salt)),
		Expiry:            expiry,
		ContractAddr:      r.contractAddr,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) IsSaltSpent(operator, salt string) (*directory.SaltResponse, error) {
	result := new(directory.SaltResponse)
	queryMsg := directory.QueryMsg{IsSaltSpent: &directory.IsSaltSpent{
		Operator: operator,
		Salt:     salt,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) GetDelegationManager() (*directory.DelegationResponse, error) {
	result := new(directory.DelegationResponse)
	queryMsg := directory.QueryMsg{
		GetDelegationManager: &directory.GetDelegationManager{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) GetOwner() (*directory.OwnerResponse, error) {
	result := new(directory.OwnerResponse)
	queryMsg := directory.QueryMsg{
		GetOwner: &directory.GetOwner{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) GetOperatorBVSRegistrationTypeHash() (*directory.RegistrationTypeHashResponse, error) {
	result := new(directory.RegistrationTypeHashResponse)
	queryMsg := directory.QueryMsg{
		GetOperatorBVSRegistrationTypeHash: &directory.GetOperatorBVSRegistrationTypeHash{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) GetDomainTypeHash() (*directory.DomainTypeHashResponse, error) {
	result := new(directory.DomainTypeHashResponse)
	queryMsg := directory.QueryMsg{
		GetDomainTypeHash: &directory.GetDomainTypeHash{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) GetDomainName() (*directory.DomainNameResponse, error) {
	result := new(directory.DomainNameResponse)
	queryMsg := directory.QueryMsg{
		GetDomainName: &directory.GetDomainName{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) GetBvsInfo(bvsHash string) (*directory.BVSInfoResponse, error) {
	result := new(directory.BVSInfoResponse)
	queryMsg := directory.QueryMsg{GetBvsInfo: &directory.GetBvsInfo{BvsHash: bvsHash}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(r.contractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (r *Directory) newExecuteOptions(contractAddr string, executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: r.gasAdjustment,
		GasPrice:      r.gasPrice,
		Gas:           r.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (r *Directory) newQueryOptions(contractAddr string, queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: contractAddr,
		QueryMsg:     queryMsg,
	}
}
