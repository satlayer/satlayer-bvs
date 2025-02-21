package api

import (
	"context"
	"encoding/base64"
	"encoding/json"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	slashmanager "github.com/satlayer/satlayer-bvs/bvs-cw/slash-manager"
)

type SlashManager interface {
	WithGasAdjustment(gasAdjustment float64) SlashManager
	WithGasPrice(gasPrice sdktypes.DecCoin) SlashManager
	WithGasLimit(gasLimit uint64) SlashManager

	BindClient(string)

	SubmitSlashRequest(ctx context.Context, slashDetails slashmanager.SubmitSlashRequestSlashDetails, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error)
	ExecuteSlashRequest(ctx context.Context, slashHash string, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error)
	CancelSlashRequest(ctx context.Context, slashHash string) (*coretypes.ResultTx, error)
	SetMinimalSlashSignature(ctx context.Context, minimalSignature int64) (*coretypes.ResultTx, error)
	SetSlasher(ctx context.Context, slasher string, value bool) (*coretypes.ResultTx, error)
	SetSlasherValidator(ctx context.Context, validators []string, values []bool) (*coretypes.ResultTx, error)
	SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)
	SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error)

	GetSlashDetails(slashHash string) (*slashmanager.SlashDetailsResponse, error)
	IsValidator(validator string) (*slashmanager.ValidatorResponse, error)
	GetMinimalSlashSignature() (*slashmanager.MinimalSlashSignatureResponse, error)
	CalculateSlashHash(sender string, slashDetails slashmanager.CalculateSlashHashSlashDetails, validatorsPublicKeys []cryptotypes.PubKey) (*slashmanager.CalculateSlashHashResponse, error)
}

type slashManagerImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func NewSlashManager(chainIO io.ChainIO, contractAddr string) SlashManager {
	return &slashManagerImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      2000000,
	}
}

func (r *slashManagerImpl) WithGasAdjustment(gasAdjustment float64) SlashManager {
	r.gasAdjustment = gasAdjustment
	return r
}

func (r *slashManagerImpl) WithGasPrice(gasPrice sdktypes.DecCoin) SlashManager {
	r.gasPrice = gasPrice
	return r
}

func (r *slashManagerImpl) WithGasLimit(gasLimit uint64) SlashManager {
	r.gasLimit = gasLimit
	return r
}

func (r *slashManagerImpl) BindClient(contractAddress string) {
	r.executeOptions = &types.ExecuteOptions{
		ContractAddr:  contractAddress,
		ExecuteMsg:    []byte{},
		Funds:         "",
		GasAdjustment: r.gasAdjustment,
		GasPrice:      r.gasPrice,
		Gas:           r.gasLimit,
		Memo:          "test tx",
		Simulate:      true,
	}

	r.queryOptions = &types.QueryOptions{
		ContractAddr: contractAddress,
		QueryMsg:     []byte{},
	}
}

func (r *slashManagerImpl) SubmitSlashRequest(ctx context.Context, slashDetails slashmanager.SubmitSlashRequestSlashDetails, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error) {
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

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) ExecuteSlashRequest(ctx context.Context, slashHash string, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error) {
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

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) CancelSlashRequest(ctx context.Context, slashHash string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		CancelSlashRequest: &slashmanager.CancelSlashRequest{
			SlashHash: slashHash,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) SetMinimalSlashSignature(ctx context.Context, minimalSignature int64) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetMinimalSlashSignature: &slashmanager.SetMinimalSlashSignature{
			MinimalSignature: minimalSignature,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) SetSlasher(ctx context.Context, slasher string, value bool) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetSlasher: &slashmanager.SetSlasher{
			Slasher: slasher,
			Value:   value,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) SetSlasherValidator(ctx context.Context, validators []string, values []bool) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetSlasherValidator: &slashmanager.SetSlasherValidator{
			Validators: validators,
			Values:     values,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetDelegationManager: &slashmanager.SetDelegationManager{
			NewDelegationManager: newDelegationManager,
		},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		Pause: &slashmanager.Pause{},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		Unpause: &slashmanager.Unpause{},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetPauser: &slashmanager.SetPauser{NewPauser: newPauser},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetUnpauser: &slashmanager.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetStrategyManager: &slashmanager.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		TransferOwnership: &slashmanager.TransferOwnership{NewOwner: newOwner},
	}
	return r.execute(ctx, executeMsg)
}

func (r *slashManagerImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.executeOptions).ExecuteMsg = msgBytes
	return r.io.SendTransaction(ctx, *r.executeOptions)
}

func (r *slashManagerImpl) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*r.queryOptions).QueryMsg = msgBytes
	return r.io.QueryContract(*r.queryOptions)
}

func (r *slashManagerImpl) GetSlashDetails(slashHash string) (*slashmanager.SlashDetailsResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		GetSlashDetails: &slashmanager.GetSlashDetails{
			SlashHash: slashHash,
		},
	}

	resp, err := r.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result slashmanager.SlashDetailsResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *slashManagerImpl) IsValidator(validator string) (*slashmanager.ValidatorResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		IsValidator: &slashmanager.IsValidator{
			Validator: validator,
		},
	}

	resp, err := r.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result slashmanager.ValidatorResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *slashManagerImpl) GetMinimalSlashSignature() (*slashmanager.MinimalSlashSignatureResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		GetMinimalSlashSignature: &slashmanager.GetMinimalSlashSignature{},
	}

	resp, err := r.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result slashmanager.MinimalSlashSignatureResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (r *slashManagerImpl) CalculateSlashHash(
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

	resp, err := r.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result slashmanager.CalculateSlashHashResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}
