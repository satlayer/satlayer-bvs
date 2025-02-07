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
)

type SlashManager interface {
	WithGasAdjustment(gasAdjustment float64) SlashManager
	WithGasPrice(gasPrice sdktypes.DecCoin) SlashManager
	WithGasLimit(gasLimit uint64) SlashManager

	BindClient(string)

	SubmitSlashRequest(ctx context.Context, slashDetails types.ExecuteSlashDetails, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error)
	ExecuteSlashRequest(ctx context.Context, slashHash string, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error)
	CancelSlashRequest(ctx context.Context, slashHash string) (*coretypes.ResultTx, error)
	SetMinimalSlashSignature(ctx context.Context, minimalSignature uint64) (*coretypes.ResultTx, error)
	SetSlasher(ctx context.Context, slasher string, value bool) (*coretypes.ResultTx, error)
	SetSlasherValidator(ctx context.Context, validators []string, values []bool) (*coretypes.ResultTx, error)
	SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error)
	TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error)
	Pause(ctx context.Context) (*coretypes.ResultTx, error)
	Unpause(ctx context.Context) (*coretypes.ResultTx, error)
	SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error)
	SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error)
	SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error)

	GetSlashDetails(slashHash string) (*types.GetSlashDetailsResp, error)
	IsValidator(validator string) (*types.IsValidatorResp, error)
	GetMinimalSlashSignature() (*types.GetMinimalSlashSignatureResp, error)
	CalculateSlashHash(sender string, slashDetails types.ExecuteSlashDetails, validatorsPublicKeys []cryptotypes.PubKey) (*types.CalculateSlashHashResp, error)
}

type slashManagerImpl struct {
	io             io.ChainIO
	executeOptions *types.ExecuteOptions
	queryOptions   *types.QueryOptions
	gasAdjustment  float64
	gasPrice       sdktypes.DecCoin
	gasLimit       uint64
}

func (s *slashManagerImpl) WithGasAdjustment(gasAdjustment float64) SlashManager {
	s.gasAdjustment = gasAdjustment
	return s
}

func (s *slashManagerImpl) WithGasPrice(gasPrice sdktypes.DecCoin) SlashManager {
	s.gasPrice = gasPrice
	return s
}

func (s *slashManagerImpl) WithGasLimit(gasLimit uint64) SlashManager {
	s.gasLimit = gasLimit
	return s
}

func (s *slashManagerImpl) BindClient(contractAddress string) {
	s.executeOptions = &types.ExecuteOptions{
		ContractAddr:  contractAddress,
		ExecuteMsg:    []byte{},
		Funds:         "",
		GasAdjustment: s.gasAdjustment,
		GasPrice:      s.gasPrice,
		Gas:           s.gasLimit,
		Memo:          "test tx",
		Simulate:      true,
	}

	s.queryOptions = &types.QueryOptions{
		ContractAddr: contractAddress,
		QueryMsg:     []byte{},
	}
}

func (s *slashManagerImpl) SubmitSlashRequest(ctx context.Context, slashDetails types.ExecuteSlashDetails, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	var encodedPublicKeys []string

	for _, pubKey := range validatorsPublicKeys {
		encodedPublicKeys = append(encodedPublicKeys, base64.StdEncoding.EncodeToString(pubKey.Bytes()))
	}

	executeMsg := types.SubmitSlashRequestReq{
		SubmitSlashRequest: types.SubmitSlashRequest{
			SlashDetails:         slashDetails,
			ValidatorsPublicKeys: encodedPublicKeys,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) ExecuteSlashRequest(ctx context.Context, slashHash string, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error) {
	slashDetailsResp, err := s.GetSlashDetails(slashHash)
	if err != nil {
		return nil, err
	}

	slashDetails := slashDetailsResp.SlashDetails

	slasherAccount, err := s.io.GetCurrentAccount()
	if err != nil {
		return nil, err
	}

	var sigs []string

	for _, validatorPublicKey := range validatorsPublicKeys {
		msgHashResp, err := s.CalculateSlashHash(
			slasherAccount.GetAddress().String(),
			slashDetails,
			[]cryptotypes.PubKey{validatorPublicKey},
		)
		if err != nil {
			return nil, err
		}

		sig, err := s.io.GetSigner().Sign(msgHashResp.MessageBytes)
		if err != nil {
			return nil, err
		}

		sigs = append(sigs, sig)
	}

	var encodedPublicKeys []string
	for _, pubKey := range validatorsPublicKeys {
		encodedPublicKeys = append(encodedPublicKeys, base64.StdEncoding.EncodeToString(pubKey.Bytes()))
	}

	executeMsg := types.ExecuteSlashRequestReq{
		ExecuteSlashRequest: types.ExecuteSlashRequest{
			SlashHash:            slashHash,
			Signatures:           sigs,
			ValidatorsPublicKeys: encodedPublicKeys,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) CancelSlashRequest(ctx context.Context, slashHash string) (*coretypes.ResultTx, error) {
	executeMsg := types.CancelSlashRequestReq{
		CancelSlashRequest: types.CancelSlashRequest{
			SlashHash: slashHash,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetMinimalSlashSignature(ctx context.Context, minimalSignature uint64) (*coretypes.ResultTx, error) {
	exeuteMsg := types.SetMinimalSlashSignatureReq{
		SetMinimalSlashSignature: types.SetMinimalSlashSignature{
			MinimalSignature: minimalSignature,
		},
	}

	return s.execute(ctx, exeuteMsg)
}

func (s *slashManagerImpl) SetSlasher(ctx context.Context, slasher string, value bool) (*coretypes.ResultTx, error) {
	executeMsg := types.SetSlasherReq{
		SetSlasher: types.SetSlasher{
			Slasher: slasher,
			Value:   value,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetSlasherValidator(ctx context.Context, validators []string, values []bool) (*coretypes.ResultTx, error) {
	executeMsg := types.SetSlasherValidatorReq{
		SetSlasherValidator: types.SetSlasherValidator{
			Validators: validators,
			Values:     values,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := types.SetDelegationManagerSlashManagerReq{
		SetDelegationManager: types.SetDelegationManagerSlashManager{
			NewDelegationManager: newDelegationManager,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := types.SlashPauseReq{}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := types.SlashUnPauseReq{}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := types.SetSlashPauserReq{
		SetPauser: types.SetSlashPauser{NewPauser: newPauser},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := types.SetSlashUnpauserReq{
		SetUnpauser: types.SetSlashUnpauser{NewUnpauser: newUnpauser},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	executeMsg := types.SlashSetStrategyManagerReq{
		SetStrategyManager: types.SlashSetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := types.TransferSlashManagerOwnershipReq{TransferOwnership: types.TransferSlashManagerOwnership{NewOwner: newOwner}}
	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) execute(ctx context.Context, msg any) (*coretypes.ResultTx, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*s.executeOptions).ExecuteMsg = msgBytes
	return s.io.SendTransaction(ctx, *s.executeOptions)
}

func (s *slashManagerImpl) query(msg any) (*wasmtypes.QuerySmartContractStateResponse, error) {
	msgBytes, err := json.Marshal(msg)

	if err != nil {
		return nil, err
	}

	(*s.queryOptions).QueryMsg = msgBytes
	return s.io.QueryContract(*s.queryOptions)
}

func (s *slashManagerImpl) GetSlashDetails(slashHash string) (*types.GetSlashDetailsResp, error) {
	queryMsg := types.GetSlashDetailsReq{
		GetSlashDetails: types.GetSlashDetails{
			SlashHash: slashHash,
		},
	}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result types.GetSlashDetailsResp
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (s *slashManagerImpl) IsValidator(validator string) (*types.IsValidatorResp, error) {
	queryMsg := types.IsValidatorReq{
		IsValidator: types.IsValidator{
			Validator: validator,
		},
	}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result types.IsValidatorResp
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (s *slashManagerImpl) GetMinimalSlashSignature() (*types.GetMinimalSlashSignatureResp, error) {
	queryMsg := types.GetMinimalSlashSignatureReq{}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result types.GetMinimalSlashSignatureResp
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (s *slashManagerImpl) CalculateSlashHash(
	sender string,
	slashDetails types.ExecuteSlashDetails,
	validatorsPublicKeys []cryptotypes.PubKey,
) (*types.CalculateSlashHashResp, error) {
	var encodedPublicKeys []string

	for _, pubKey := range validatorsPublicKeys {
		encodedPublicKeys = append(encodedPublicKeys, base64.StdEncoding.EncodeToString(pubKey.Bytes()))
	}

	queryMsg := types.CalculateSlashHashReq{
		CalculateSlashHash: types.CalculateSlashHash{
			Sender:               sender,
			SlashDetails:         slashDetails,
			ValidatorsPublicKeys: encodedPublicKeys,
		},
	}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result types.CalculateSlashHashResp
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func NewSlashManagerImpl(chainIO io.ChainIO, contractAddr string) SlashManager {
	return &slashManagerImpl{
		io:            chainIO,
		gasAdjustment: 1.2,
		gasPrice:      sdktypes.NewInt64DecCoin("ubbn", 1),
		gasLimit:      2000000,
	}
}
