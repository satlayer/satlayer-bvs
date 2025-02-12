package types

import (
	"time"

	sdktypes "github.com/cosmos/cosmos-sdk/types"
)

const DefaultKeyringServiceName = "babylon"

type ExecuteOptions struct {
	ContractAddr  string           // ContractAddr: Address of the smart contract
	ExecuteMsg    []byte           // ExecuteMsg: Message to be executed, represented as a struct
	Funds         string           // Funds: Amount of funds to send to the contract, represented as a string
	GasAdjustment float64          // GasAdjustment: Gas adjustment factor for adjusting the estimated gas amount
	GasPrice      sdktypes.DecCoin // GasPrice: Gas price, represented as a string (e.g., "0.025uatom")
	Gas           uint64           // Gas: Amount of gas reserved for transaction execution
	Memo          string           // Memo: Transaction memo information
	Simulate      bool             // Simulate: Whether to simulate the transaction to estimate gas usage and set Gas accordingly
}

type QueryOptions struct {
	ContractAddr string // ContractAddr: Address of the smart contract
	QueryMsg     []byte // QueryMsg: Query message json encoding
}

type TxManagerParams struct {
	MaxRetries             int
	RetryInterval          time.Duration
	ConfirmationTimeout    time.Duration
	GasPriceAdjustmentRate string
}
