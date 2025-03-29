package cosmwasmapi

import (
	"encoding/json"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
)

type BroadcastOptions struct {
	ContractAddr  string           // ContractAddr: Address of the smart contract
	ExecuteMsg    []byte           // ExecuteMsg: Message to be executed, represented as a struct
	Funds         sdktypes.Coins   // Funds: Amount of funds to send to the contract, represented as Coins
	GasAdjustment float64          // GasAdjustment: Gas adjustment factor for adjusting the estimated gas amount
	GasPrice      sdktypes.DecCoin // GasPrice: Gas price, represented as a string (e.g., "0.02ubbn")
	Gas           uint64           // Gas: Amount of gas reserved for transaction execution
	Simulate      bool             // Simulate: Whether to simulate the transaction to estimate gas usage and set Gas accordingly
}

func DefaultBroadcastOptions() BroadcastOptions {
	return BroadcastOptions{
		Funds:         sdktypes.Coins{},
		GasAdjustment: 1.2,
		GasPrice:      sdktypes.NewInt64DecCoin("ubbn", 2000),
		Gas:           200_000,
		Simulate:      true,
	}
}

func (opts BroadcastOptions) WithContractAddr(contractAddr string) BroadcastOptions {
	opts.ContractAddr = contractAddr
	return opts
}

func (opts BroadcastOptions) WithExecuteMsg(executeMsg any) BroadcastOptions {
	executeMsgBytes, err := json.Marshal(executeMsg)
	if err != nil {
		panic(err)
	}

	opts.ExecuteMsg = executeMsgBytes
	return opts
}

func (opts BroadcastOptions) WithFunds(funds string) BroadcastOptions {
	coinFunds, err := sdktypes.ParseCoinsNormalized(funds)
	if err != nil {
		panic(err)
	}

	opts.Funds = coinFunds
	return opts
}

func (opts BroadcastOptions) WithGasAdjustment(gasAdjustment float64) BroadcastOptions {
	opts.GasAdjustment = gasAdjustment
	return opts
}

func (opts BroadcastOptions) WithGasPrice(gasPrice string) BroadcastOptions {
	coin, err := sdktypes.ParseDecCoin(gasPrice)
	if err != nil {
		panic(err)
	}
	opts.GasPrice = coin
	return opts
}

func (opts BroadcastOptions) WithGas(gas uint64) BroadcastOptions {
	opts.Gas = gas
	return opts
}

func (opts BroadcastOptions) WithSimulate(simulate bool) BroadcastOptions {
	opts.Simulate = simulate
	return opts
}
