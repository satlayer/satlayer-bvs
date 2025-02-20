package api

import (
	"context"
	"encoding/base64"
	"encoding/json"

	slashmanager "github.com/satlayer/satlayer-bvs/bvs-cw/slash-manager"

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

func (s *slashManagerImpl) SubmitSlashRequest(ctx context.Context, slashDetails slashmanager.SubmitSlashRequestSlashDetails, validatorsPublicKeys []cryptotypes.PubKey) (*coretypes.ResultTx, error) {
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
		sig, err := s.io.GetSigner().Sign(bytes)
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

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) CancelSlashRequest(ctx context.Context, slashHash string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		CancelSlashRequest: &slashmanager.CancelSlashRequest{
			SlashHash: slashHash,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetMinimalSlashSignature(ctx context.Context, minimalSignature int64) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetMinimalSlashSignature: &slashmanager.SetMinimalSlashSignature{
			MinimalSignature: minimalSignature,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetSlasher(ctx context.Context, slasher string, value bool) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetSlasher: &slashmanager.SetSlasher{
			Slasher: slasher,
			Value:   value,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetSlasherValidator(ctx context.Context, validators []string, values []bool) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetSlasherValidator: &slashmanager.SetSlasherValidator{
			Validators: validators,
			Values:     values,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetDelegationManager(ctx context.Context, newDelegationManager string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetDelegationManager: &slashmanager.SetDelegationManager{
			NewDelegationManager: newDelegationManager,
		},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) Pause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		Pause: &slashmanager.Pause{},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) Unpause(ctx context.Context) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		Unpause: &slashmanager.Unpause{},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetPauser(ctx context.Context, newPauser string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetPauser: &slashmanager.SetPauser{NewPauser: newPauser},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetUnpauser(ctx context.Context, newUnpauser string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetUnpauser: &slashmanager.SetUnpauser{NewUnpauser: newUnpauser},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) SetStrategyManager(ctx context.Context, newStrategyManager string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		SetStrategyManager: &slashmanager.SetStrategyManager{NewStrategyManager: newStrategyManager},
	}

	return s.execute(ctx, executeMsg)
}

func (s *slashManagerImpl) TransferOwnership(ctx context.Context, newOwner string) (*coretypes.ResultTx, error) {
	executeMsg := slashmanager.ExecuteMsg{
		TransferOwnership: &slashmanager.TransferOwnership{NewOwner: newOwner},
	}
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

func (s *slashManagerImpl) GetSlashDetails(slashHash string) (*slashmanager.SlashDetailsResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		GetSlashDetails: &slashmanager.GetSlashDetails{
			SlashHash: slashHash,
		},
	}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result slashmanager.SlashDetailsResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (s *slashManagerImpl) IsValidator(validator string) (*slashmanager.ValidatorResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		IsValidator: &slashmanager.IsValidator{
			Validator: validator,
		},
	}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result slashmanager.ValidatorResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (s *slashManagerImpl) GetMinimalSlashSignature() (*slashmanager.MinimalSlashSignatureResponse, error) {
	queryMsg := slashmanager.QueryMsg{
		GetMinimalSlashSignature: &slashmanager.GetMinimalSlashSignature{},
	}

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result slashmanager.MinimalSlashSignatureResponse
	if err := json.Unmarshal(resp.Data, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

func (s *slashManagerImpl) CalculateSlashHash(
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

	resp, err := s.query(queryMsg)
	if err != nil {
		return nil, err
	}

	var result slashmanager.CalculateSlashHashResponse
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
