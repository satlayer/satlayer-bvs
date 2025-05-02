package database

import (
	"fmt"

	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database/utils"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"
)

// SaveWASMExecuteContract allows to store one WASM execute contract.
func (db *DB) SaveWASMExecuteContract(wasmExecuteContract types.WASMExecuteContract) error {
	return db.SaveWASMExecuteContracts([]types.WASMExecuteContract{wasmExecuteContract})
}

// SaveWasmContracts allows to store WASM contract slice.
func (db *DB) SaveWASMExecuteContracts(executeContracts []types.WASMExecuteContract) error {
	paramsNumber := 9
	slices := utils.SplitWASMExecuteContracts(executeContracts, paramsNumber)

	for _, contracts := range slices {
		if len(contracts) == 0 {
			continue
		}

		err := db.saveWASMExecuteContracts(paramsNumber, contracts)
		if err != nil {
			return fmt.Errorf("failed to store WASM contracts: %s", err)
		}
	}

	return nil
}

func (db *DB) saveWASMExecuteContracts(paramNumber int, executeContracts []types.WASMExecuteContract) error {
	stmt := `
INSERT INTO wasm_execute_contract 
(sender, contract_address, execute_contract_message, message_type, wasm_event, custom_wasm_event, executed_at, height, tx_hash) 
VALUES `

	var args []any
	for i, executeContract := range executeContracts {
		ii := i * paramNumber
		stmt += fmt.Sprintf("($%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d),",
			ii+1, ii+2, ii+3, ii+4, ii+5, ii+6, ii+7, ii+8, ii+9)
		args = append(args, executeContract.Sender, executeContract.ContractAddress, string(executeContract.ExecuteContractMessage),
			executeContract.MessageType, string(executeContract.WASMEvent), string(executeContract.CustomWASMEvent),
			executeContract.ExecutedAt, executeContract.Height, executeContract.TxHash)
	}

	// Remove trailing ","
	stmt = stmt[:len(stmt)-1]

	stmt += ` ON CONFLICT (height, tx_hash) DO NOTHING`

	_, err := db.SQL.Exec(stmt, args...)
	if err != nil {
		return fmt.Errorf("failed to save WASM execute contracts: %s", err)
	}

	return nil
}
