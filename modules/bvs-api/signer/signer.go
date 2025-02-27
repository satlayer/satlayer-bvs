package signer

import (
	"encoding/base64"
	"errors"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/crypto"
	sdksecp256k1 "github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	dcrdsecp256k1 "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa"

	"github.com/satlayer/satlayer-bvs/bvs-api/utils"
)

type Signer struct {
	ClientCtx client.Context
	validator MsgValidator
}

func NewSigner(clientCtx client.Context) *Signer {
	return &Signer{
		ClientCtx: clientCtx,
		validator: &DefaultMsgValidator{},
	}
}

func (s *Signer) BuildAndSignTx(gasAdjustment float64, gasPrice sdktypes.DecCoin, maxGas uint64, memo string, simulate bool, msgs ...sdktypes.Msg) (sdktypes.Tx, error) {
	txBuilder, txf, err := s.BuildUnsignedTx(gasAdjustment, gasPrice, maxGas, memo, simulate, msgs...)
	if err != nil {
		return nil, err
	}

	// Sign the transaction
	if err = tx.Sign(s.ClientCtx.CmdContext, txf, s.ClientCtx.GetFromName(), txBuilder, true); err != nil {
		return nil, err
	}

	return txBuilder.GetTx(), nil
}

func (s *Signer) BuildUnsignedTx(gasAdjustment float64, gasPrice sdktypes.DecCoin, maxGas uint64, memo string, simulate bool, msgs ...sdktypes.Msg) (client.TxBuilder, tx.Factory, error) {
	msgs, err := s.CheckMsg(msgs...)
	if err != nil {
		return nil, tx.Factory{}, err
	}
	txf, err := s.setFactory(gasAdjustment, gasPrice, maxGas, memo, simulate).Prepare(s.ClientCtx)
	if err != nil {
		return nil, tx.Factory{}, err
	}
	// whether to simulate gas calculations
	if txf.SimulateAndExecute() {
		_, adjusted, err := tx.CalculateGas(s.ClientCtx, txf, msgs...)
		if err != nil {
			return nil, tx.Factory{}, err
		}
		if adjusted > maxGas {
			adjusted = maxGas
		}
		txf = txf.WithGas(adjusted)
	}
	// Build an unsigned transaction
	txBuilder, err := txf.BuildUnsignedTx(msgs...)
	if err != nil {
		return nil, tx.Factory{}, err
	}
	return txBuilder, txf, nil
}

func (s *Signer) setFactory(gasAdjustment float64, gasPrice sdktypes.DecCoin, maxGas uint64, memo string, simulate bool) tx.Factory {
	txf := tx.Factory{}.
		WithChainID(s.ClientCtx.ChainID).                   // Set the chain ID to specify the blockchain the transaction will be sent to
		WithKeybase(s.ClientCtx.Keyring).                   // Set up the keystore, using the keystore instance configured in client.Context
		WithTxConfig(s.ClientCtx.TxConfig).                 // Set up transaction configurations to specify how transactions are encoded and decoded
		WithAccountRetriever(s.ClientCtx.AccountRetriever). // Set up an account retriever to obtain account information from the chain
		WithSimulateAndExecute(simulate).                   // Set up simulation and execution, first simulate the transaction to obtain the estimated gas, and then execute the transaction
		WithSignMode(signing.SignMode_SIGN_MODE_DIRECT).    // Set the signature mode to SIGN_MODE_DIRECT and use direct signature mode
		WithGas(maxGas).                                    // Set the gas limit
		WithGasAdjustment(gasAdjustment).                   // Set the gas adjustment factor to 1.3 to increase the estimated gas to ensure transaction success
		WithGasPrices(gasPrice.String()).                   // Set the gas price to 0.1uosmo and specify the token and amount to pay for the transaction fee
		WithFromName(s.ClientCtx.FromName).                 // Set from name
		WithMemo(memo)                                      // Set the transaction note to an empty string, and you can add custom note information
	return txf
}

// CheckMsg validates the given messages
func (s *Signer) CheckMsg(msgs ...sdktypes.Msg) ([]sdktypes.Msg, error) {
	for _, msg := range msgs {
		if err := s.validator.ValidateMsg(msg); err != nil {
			return nil, err
		}
	}
	return msgs, nil
}

func (s *Signer) Sign(msgHash []byte) (string, error) {
	decryptPrivKeyPwd, err := utils.GenerateRandomString(16)
	if err != nil {
		return "", err
	}
	armor, err := s.ClientCtx.Keyring.ExportPrivKeyArmor(s.ClientCtx.FromName, decryptPrivKeyPwd)
	if err != nil {
		return "", err
	}
	// decrypt private key
	privKey, _, err := crypto.UnarmorDecryptPrivKey(armor, decryptPrivKeyPwd)
	if err != nil {
		return "", err
	}
	// convert the private key to secp256k1.PrivKey type
	secp256k1PrivKey, ok := privKey.(*sdksecp256k1.PrivKey)
	if !ok {
		return "", errors.New("invalid secp256k1 privkey")
	}
	var secKey = dcrdsecp256k1.PrivKeyFromBytes(secp256k1PrivKey.Bytes())
	signature := ecdsa.SignCompact(secKey, msgHash, false)
	// remove the recovery bit and convert signature to base64 string
	return base64.StdEncoding.EncodeToString(signature[1:]), nil
}

func (s *Signer) SignByKeyName(msgHash []byte, keyName string) (string, error) {
	decryptPrivKeyPwd, err := utils.GenerateRandomString(16)
	if err != nil {
		return "", err
	}
	armor, err := s.ClientCtx.Keyring.ExportPrivKeyArmor(keyName, decryptPrivKeyPwd)
	if err != nil {
		return "", err
	}
	// decrypt private key
	privKey, _, err := crypto.UnarmorDecryptPrivKey(armor, decryptPrivKeyPwd)
	if err != nil {
		return "", err
	}
	// convert the private key to secp256k1.PrivKey type
	secp256k1PrivKey, ok := privKey.(*sdksecp256k1.PrivKey)
	if !ok {
		return "", errors.New("invalid secp256k1 privkey")
	}
	var secKey = dcrdsecp256k1.PrivKeyFromBytes(secp256k1PrivKey.Bytes())
	signature := ecdsa.SignCompact(secKey, msgHash, false)
	// remove the recovery bit and convert signature to base64 string
	return base64.StdEncoding.EncodeToString(signature[1:]), nil
}

// SetMsgValidator allows setting a custom message validator
func (s *Signer) SetMsgValidator(v MsgValidator) {
	s.validator = v
}
