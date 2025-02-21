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

func (i *BvsDirectory) WithGasAdjustment(gasAdjustment float64) *BvsDirectory {
	i.gasAdjustment = gasAdjustment
	return i
}

func (i *BvsDirectory) WithGasPrice(gasPrice sdktypes.DecCoin) *BvsDirectory {
	i.gasPrice = gasPrice
	return i
}

func (i *BvsDirectory) WithGasLimit(gasLimit uint64) *BvsDirectory {
	i.gasLimit = gasLimit
	return i
}

func (i *BvsDirectory) RegisterBVS(ctx context.Context, bvsContract string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{RegisterBVS: &directory.RegisterBVS{
		BvsContract: bvsContract,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "RegisterBVS")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) RegisterOperator(ctx context.Context, operator string, publicKey cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	nodeStatus, err := i.io.QueryNodeStatus(context.Background())
	if err != nil {
		return nil, err
	}
	expiry := nodeStatus.SyncInfo.LatestBlockTime.Unix() + 1000

	randomStr, err := utils.GenerateRandomString(16)
	if err != nil {
		return nil, err
	}
	salt := "salt" + randomStr

	msgHashResp, err := i.CalculateDigestHash(publicKey, operator, salt, expiry)
	if err != nil {
		return nil, err
	}
	bytes, err := base64.StdEncoding.DecodeString(msgHashResp.DigestHash)
	if err != nil {
		return nil, err
	}
	sig, err := i.io.GetSigner().Sign(bytes)
	if err != nil {
		return nil, err
	}
	executeMsg := directory.ExecuteMsg{RegisterOperatorToBVS: &directory.RegisterOperatorToBVS{
		Operator:     operator,
		PublicKey:    base64.StdEncoding.EncodeToString(publicKey.Bytes()),
		ContractAddr: i.contractAddr,
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
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "RegisterOperator")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) DeregisterOperator(ctx context.Context, operator string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{DeregisterOperatorFromBVS: &directory.DeregisterOperatorFromBVS{Operator: operator}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "DeregisterOperator")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) UpdateMetadataURI(ctx context.Context, metadataURI string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{UpdateBVSMetadataURI: &directory.UpdateBVSMetadataURI{MetadataURI: metadataURI}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "UpdateMetadataURI")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) CancelSalt(ctx context.Context, salt string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{CancelSalt: &directory.CancelSalt{Salt: base64.StdEncoding.EncodeToString([]byte(salt))}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "CancelSalt")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{TransferOwnership: &directory.TransferOwnership{NewOwner: newOwner}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "TransferOwnership")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{Pause: &directory.Pause{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "Pause")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{Unpause: &directory.Unpause{}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "Unpause")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetPauser: &directory.SetPauser{NewPauser: newPauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "SetPauser")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetUnpauser: &directory.SetUnpauser{NewUnpauser: newUnpauser}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "SetUnpauser")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) SetDelegationManager(ctx context.Context, delegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{SetDelegationManager: &directory.SetDelegationManager{DelegationManager: delegationManager}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := i.newExecuteOptions(i.contractAddr, executeMsgBytes, "SetDelegationManager")

	return i.io.SendTransaction(ctx, executeOptions)
}

func (i *BvsDirectory) QueryOperator(bvs, operator string) (*directory.OperatorStatusResponse, error) {
	result := new(directory.OperatorStatusResponse)
	queryMsg := directory.QueryMsg{GetOperatorStatus: &directory.GetOperatorStatus{
		Operator: operator,
		Bvs:      bvs,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) CalculateDigestHash(operatorPublicKey cryptotypes.PubKey, bvs, salt string, expiry int64) (*directory.DigestHashResponse, error) {
	result := new(directory.DigestHashResponse)
	queryMsg := &directory.QueryMsg{CalculateDigestHash: &directory.CalculateDigestHash{
		OperatorPublicKey: base64.StdEncoding.EncodeToString(operatorPublicKey.Bytes()),
		Bvs:               bvs,
		Salt:              base64.StdEncoding.EncodeToString([]byte(salt)),
		Expiry:            expiry,
		ContractAddr:      i.contractAddr,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) IsSaltSpent(operator, salt string) (*directory.SaltResponse, error) {
	result := new(directory.SaltResponse)
	queryMsg := directory.QueryMsg{IsSaltSpent: &directory.IsSaltSpent{
		Operator: operator,
		Salt:     salt,
	}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) GetDelegationManager() (*directory.DelegationResponse, error) {
	result := new(directory.DelegationResponse)
	queryMsg := directory.QueryMsg{
		GetDelegationManager: &directory.GetDelegationManager{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) GetOwner() (*directory.OwnerResponse, error) {
	result := new(directory.OwnerResponse)
	queryMsg := directory.QueryMsg{
		GetOwner: &directory.GetOwner{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) GetOperatorBVSRegistrationTypeHash() (*directory.RegistrationTypeHashResponse, error) {
	result := new(directory.RegistrationTypeHashResponse)
	queryMsg := directory.QueryMsg{
		GetOperatorBVSRegistrationTypeHash: &directory.GetOperatorBVSRegistrationTypeHash{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) GetDomainTypeHash() (*directory.DomainTypeHashResponse, error) {
	result := new(directory.DomainTypeHashResponse)
	queryMsg := directory.QueryMsg{
		GetDomainTypeHash: &directory.GetDomainTypeHash{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) GetDomainName() (*directory.DomainNameResponse, error) {
	result := new(directory.DomainNameResponse)
	queryMsg := directory.QueryMsg{
		GetDomainName: &directory.GetDomainName{},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) GetBVSInfo(bvsHash string) (*directory.BVSInfoResponse, error) {
	result := new(directory.BVSInfoResponse)
	queryMsg := directory.QueryMsg{GetBVSInfo: &directory.GetBVSInfo{BvsHash: bvsHash}}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := i.newQueryOptions(i.contractAddr, queryMsgBytes)
	resp, err := i.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(resp.Data, result); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *BvsDirectory) newExecuteOptions(contractAddr string, executeMsg []byte, memo string) types.ExecuteOptions {
	return types.ExecuteOptions{
		ContractAddr:  contractAddr,
		ExecuteMsg:    executeMsg,
		Funds:         "",
		GasAdjustment: i.gasAdjustment,
		GasPrice:      i.gasPrice,
		Gas:           i.gasLimit,
		Memo:          memo,
		Simulate:      true,
	}
}

func (i *BvsDirectory) newQueryOptions(contractAddr string, queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: contractAddr,
		QueryMsg:     queryMsg,
	}
}
