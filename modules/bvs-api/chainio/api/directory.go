package api

import (
	"context"
	"encoding/json"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/bvs-cw/directory"
)

type Directory struct {
	io            io.ChainIO
	ContractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewDirectory(chainIO io.ChainIO, contractAddr string) *Directory {
	return &Directory{
		io:            chainIO,
		ContractAddr:  contractAddr,
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

func (r *Directory) ServiceRegister(ctx context.Context, metadata directory.ServiceMetadata) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{ServiceRegister: &directory.ServiceRegister{
		Metadata: metadata,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "Service Register")
	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) ServiceUpdateMetadata(ctx context.Context, metadata directory.ServiceMetadata) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{ServiceUpdateMetadata: &metadata}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "Service Update Metadata")
	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) ServiceRegisterOperator(ctx context.Context, operator string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{ServiceRegisterOperator: &directory.ServiceRegisterOperator{
		Operator: operator,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "Service Register Operator")
	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) ServiceDeregisterOperator(ctx context.Context, operator string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{ServiceDeregisterOperator: &directory.ServiceDeregisterOperator{
		Operator: operator,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "Service Deregister Operator")
	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) OperatorRegisterService(ctx context.Context, service string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{OperatorRegisterService: &directory.OperatorRegisterService{
		Service: service,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "Operator Register Service")
	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) OperatorDeregisterService(ctx context.Context, service string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{OperatorDeregisterService: &directory.OperatorDeregisterService{
		Service: service,
	}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "Operator Deregister Service")
	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{TransferOwnership: &directory.TransferOwnership{NewOwner: newOwner}}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "TransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) SetRouting(ctx context.Context, delegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := directory.ExecuteMsg{
		SetRouting: &directory.SetRouting{
			DelegationManager: delegationManager,
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(r.ContractAddr, executeMsgBytes, "SetDelegationManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *Directory) QueryStatus(operator, service string) (*directory.StatusResponse, error) {
	queryMsg := directory.QueryMsg{
		Status: directory.Status{
			Operator: operator,
			Service:  service,
		},
	}
	queryMsgBytes, err := queryMsg.Marshal()
	if err != nil {
		return nil, err
	}

	queryOptions := r.newQueryOptions(r.ContractAddr, queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	status, err := directory.UnmarshalStatusResponse(resp.Data)
	return &status, err
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
