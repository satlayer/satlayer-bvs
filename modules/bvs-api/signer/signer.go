package signer

import (
	"encoding/base64"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

type Signer struct {
	ClientCtx client.Context
}

func NewSigner(clientCtx client.Context) *Signer {
	return &Signer{
		ClientCtx: clientCtx,
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
	msgs, err := s.checkMsg(msgs...)
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

func (s *Signer) checkMsg(msgs ...sdktypes.Msg) ([]sdktypes.Msg, error) {
	for _, msg := range msgs {
		m, ok := msg.(sdktypes.HasValidateBasic)
		if !ok {
			continue
		}

		if err := m.ValidateBasic(); err != nil {
			return nil, err
		}
	}
	return msgs, nil
}

func (s *Signer) Sign(msgHash []byte) (string, error) {
	signature, _, err := s.ClientCtx.Keyring.Sign(
		s.ClientCtx.FromName,
		msgHash,
		signing.SignMode_SIGN_MODE_DIRECT,
	)
	if err != nil {
		return "", err
	}

	return base64.StdEncoding.EncodeToString(signature), nil
}

func (s *Signer) SignByKeyName(msgHash []byte, keyName string) (string, error) {
	signature, _, err := s.ClientCtx.Keyring.Sign(
		keyName,
		msgHash,
		signing.SignMode_SIGN_MODE_DIRECT,
	)
	if err != nil {
		return "", err
	}

	return base64.StdEncoding.EncodeToString(signature), nil
}
