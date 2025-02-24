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
	slashmanager "github.com/satlayer/satlayer-bvs/bvs-cw/slash-manager"
)

type SlashManager struct {
	io            io.ChainIO
	contractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func NewSlashManager(chainIO io.ChainIO, contractAddr string) *SlashManager {
	// TODO(fuxingloh): unused ContractAddr
	return &SlashManager{
		io:            chainIO,
		contractAddr:  contractAddr,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      2000000,
	}
}

func (r *SlashManager) WithGasAdjustment(gasAdjustment float64) *SlashManager {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *SlashManager) WithGasPrice(gasPrice sdktypes.DecCoin) *SlashManager {
	r.gasPrice = gasPrice
	return r
}

func (r *SlashManager) WithGasLimit(gasLimit uint64) *SlashManager {
	r.gasLimit = gasLimit
	return r
}

func (r *SlashManager) SubmitSlashRequest(ctx context.Context, slashDetails slashmanager.SubmitSlashRequestSlashDetails, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	var encodedPublicKeys []string

	for _, pubKey := range validatorsPublicKeys {
		encodedPublicKeys = append(encodedPublicKeys, base64.StdEncoding.EncodeToString(pubKey.Bytes()))
	}

	executeMsg := slashmanager.ExecuteMsg{
		SubmitSlashRequest: &slashmanager.SubmitSlashRequest{
			SlashDetails:         slashDetails,
			ValidatorsPublicKeys: encodedPublicKeys,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SubmitSlashRequest")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) ExecuteSlashRequest(ctx context.Context, slashHash string, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	slashDetailsResp, err := r.GetSlashDetails(slashHash)
	if err != nil {
		return nil, err
	}

	slashDetails := slashDetailsResp.SlashDetails

	slasherAccount, err := r.io.GetCurrentAccount()
	if err != nil {
		return nil, err
	}

	var sigs []string

	for _, validatorPublicKey := range validatorsPublicKeys {
		msgHashResp, err := r.CalculateSlashHash(
			slasherAccount.GetAddress().String(),
			slashmanager.CalculateSlashHashSlashDetails{
				Slasher:        slashDetails.Slasher,
				Operator:       slashDetails.Operator,
				Share:          slashDetails.Share,
				SlashSignature: slashDetails.SlashSignature,
				SlashValidator: slashDetails.SlashValidator,
				Reason:         slashDetails.Reason,
				StartTime:      slashDetails.StartTime,
				EndTime:        slashDetails.EndTime,
				Status:         slashDetails.Status,
			},
			[]cryptotypes.PubKey{validatorPublicKey},
		)
		if err != nil {
			return nil, err
		}

		// convert from int64 into byte, see SL-184
		bytes := make([]byte, len(msgHashResp.MessageBytes))
		for i, v := range msgHashResp.MessageBytes {
			bytes[i] = byte(v)
		}
		sig, err := r.io.GetSigner().Sign(bytes)
		if err != nil {
			return nil, err
		}

		sigs = append(sigs, sig)
	}

	var encodedPublicKeys []string
	for _, pubKey := range validatorsPublicKeys {
		encodedPublicKeys = append(encodedPublicKeys, base64.StdEncoding.EncodeToString(pubKey.Bytes()))
	}

	executeMsg := slashmanager.ExecuteMsg{
		ExecuteSlashRequest: &slashmanager.ExecuteSlashRequest{
			SlashHash:            slashHash,
			Signatures:           sigs,
			ValidatorsPublicKeys: encodedPublicKeys,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "ExecuteSlashRequest")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) CancelSlashRequest(ctx context.Context, slashHash string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		CancelSlashRequest: &slashmanager.CancelSlashRequest{
			SlashHash: slashHash,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "CancelSlashRequest")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) SetMinimalSlashSignature(ctx context.Context, minimalSignature int64) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetMinimalSlashSignature: &slashmanager.SetMinimalSlashSignature{
			MinimalSignature: minimalSignature,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetMinimalSlashSignature")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) SetSlasher(ctx context.Context, slasher string, value bool) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetSlasher: &slashmanager.SetSlasher{
			Slasher: slasher,
			Value:   value,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetSlasher")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) SetSlasherValidator(ctx context.Context, validators []string, values []bool) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetSlasherValidator: &slashmanager.SetSlasherValidator{
			Validators: validators,
			Values:     values,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetSlasherValidator")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetDelegationManager: &slashmanager.SetDelegationManager{
			NewDelegationManager: newDelegationManager,
		},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetDelegationManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		Pause: &slashmanager.Pause{},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Pause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		Unpause: &slashmanager.Unpause{},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "Unpause")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetPauser: &slashmanager.SetPauser{NewPauser: newPauser},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetPauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetUnpauser: &slashmanager.SetUnpauser{NewUnpauser: newUnpauser},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetUnpauser")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetStrategyManager: &slashmanager.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "SetStrategyManager")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		TransferOwnership: &slashmanager.TransferOwnership{NewOwner: newOwner},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		return nil, err
	}
	executeOptions := r.newExecuteOptions(executeMsgBytes, "TransferOwnership")

	return r.io.SendTransaction(ctx, executeOptions)
}

func (r *SlashManager) GetSlashDetails(slashHash string) (*slashmanager.SlashDetailsResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		GetSlashDetails: &slashmanager.GetSlashDetails{
			SlashHash: slashHash,
		},
	}
	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	var result slashmanager.SlashDetailsResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *SlashManager) IsValidator(validator string) (*slashmanager.ValidatorResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		IsValidator: &slashmanager.IsValidator{
			Validator: validator,
		},
	}

	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	var result slashmanager.ValidatorResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *SlashManager) GetMinimalSlashSignature() (*slashmanager.MinimalSlashSignatureResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		GetMinimalSlashSignature: &slashmanager.GetMinimalSlashSignature{},
	}

	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	var result slashmanager.MinimalSlashSignatureResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *SlashManager) CalculateSlashHash(
	sender string,
	slashDetails slashmanager.CalculateSlashHashSlashDetails,
	validatorsPublicKeys []cryptotypes.PubKey,
) (*slashmanager.CalculateSlashHashResponse, error) {
	var encodedPublicKeys []string

	for _, pubKey := range validatorsPublicKeys {
		encodedPublicKeys = append(encodedPublicKeys, base64.StdEncoding.EncodeToString(pubKey.Bytes()))
	}

	queryMsg := slashmanager.QueryMsg{
		CalculateSlashHash: &slashmanager.CalculateSlashHash{
			Sender:               sender,
			SlashDetails:         slashDetails,
			ValidatorsPublicKeys: encodedPublicKeys,
		},
	}

	queryMsgBytes, err := json.Marshal(queryMsg)
	if err != nil {
		return nil, err
	}
	queryOptions := r.newQueryOptions(queryMsgBytes)
	resp, err := r.io.QueryContract(queryOptions)
	if err != nil {
		return nil, err
	}

	var result slashmanager.CalculateSlashHashResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *SlashManager) newExecuteOptions(executeMsg []byte, memo string) types.ExecuteOptions {
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

func (r *SlashManager) newQueryOptions(queryMsg []byte) types.QueryOptions {
	return types.QueryOptions{
		ContractAddr: r.contractAddr,
		QueryMsg:     queryMsg,
	}
}
