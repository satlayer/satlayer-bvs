package database

import (
	"fmt"

	junotypes "github.com/forbole/juno/v6/types"
	"github.com/lib/pq"

	dbtypes "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database/types"
	dbutils "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database/utils"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"
)

// SaveWASMParams allows to store the WASM params for genesis file state.
func (db *DB) SaveWASMParams(params types.WASMParams) error {
	stmt := `
INSERT INTO wasm_params(code_upload_access, instantiate_default_permission, height) 
VALUES ($1, $2, $3) 
ON CONFLICT (one_row_id) DO UPDATE 
	SET code_upload_access = excluded.code_upload_access, 
		instantiate_default_permission = excluded.instantiate_default_permission, 
WHERE wasm_params.height <= excluded.height
`
	accessConfig := dbtypes.NewDBAccessConfig(params.CodeUploadAccess)
	cfgValue, _ := accessConfig.Value()

	_, err := db.SQL.Exec(stmt,
		cfgValue, params.InstantiateDefaultPermission, params.Height,
	)
	if err != nil {
		return fmt.Errorf("failed to save wasm params: %s", err)
	}

	return nil
}

// SaveWASMCode allows to store a single WASM code
func (db *DB) SaveWASMCode(wasmCode types.WASMCode) error {
	return db.SaveWASMCodes([]types.WASMCode{wasmCode})
}

// SaveWASMCodes allows to store the wasm code slice
func (db *DB) SaveWASMCodes(wasmCodes []types.WASMCode) error {
	stmt := `
INSERT INTO wasm_code(sender, byte_code, instantiate_permission, code_id, height) 
VALUES `

	var args []any
	for i, code := range wasmCodes {
		ii := i * 5

		var accessConfig dbtypes.DBAccessConfig
		if code.InstantiatePermission != nil {
			accessConfig = dbtypes.NewDBAccessConfig(code.InstantiatePermission)
		}

		cfgValue, _ := accessConfig.Value()

		stmt += fmt.Sprintf("($%d, $%d, $%d, $%d, $%d),", ii+1, ii+2, ii+3, ii+4, ii+5)
		args = append(args, code.Sender, code.WASMByteCode, cfgValue, code.CodeID, code.Height)
	}

	// Remove trailing ","
	stmt = stmt[:len(stmt)-1]

	stmt += `
	ON CONFLICT (code_id) DO UPDATE 
		SET sender = excluded.sender,
			byte_code = excluded.byte_code,
			instantiate_permission = excluded.instantiate_permission,
			height = excluded.height
	WHERE wasm_code.height <= excluded.height`

	_, err := db.SQL.Exec(stmt, args...)
	if err != nil {
		return fmt.Errorf("failed to save WASM byte code: %s", err)
	}

	return nil
}

// SaveWASMInstantiateContracts allows to store the WASM instantiate contract slice.
func (db *DB) SaveWASMInstantiateContracts(contracts []types.WASMInstantiateContract) error {
	paramsNumber := 13
	slices := dbutils.SplitWASMInstantiateContracts(contracts, paramsNumber)

	for _, contracts = range slices {
		if len(contracts) == 0 {
			continue
		}

		err := db.saveWASMInstantiateContracts(paramsNumber, contracts)
		if err != nil {
			return fmt.Errorf("failed to store WASM contracts: %s", err)
		}
	}

	return nil
}

func (db *DB) saveWASMInstantiateContracts(paramsNumber int, wasmContracts []types.WASMInstantiateContract) error {
	stmt := `
INSERT INTO wasm_instantiate_contract 
(sender, creator, admin, code_id, label, instantiate_contract_message, contract_address, wasm_event, custom_wasm_event, 
contract_info_extension, contract_states, funds, instantiated_at, height, tx_hash) 
VALUES `

	// only add new one, shouldn't be repeated
	var args []any
	for i, contract := range wasmContracts {
		ii := i * paramsNumber
		stmt += fmt.Sprintf("($%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d),",
			ii+1, ii+2, ii+3, ii+4, ii+5, ii+6, ii+7, ii+8, ii+9, ii+10, ii+11, ii+12, ii+13, ii+14, ii+15)
		args = append(args,
			contract.Sender, contract.Creator, contract.Admin, contract.CodeID, contract.Label, string(contract.InstantiateContractMsg),
			contract.ContractAddress, string(contract.WASMEvent), string(contract.CustomWASMEvent), contract.ContractInfoExtension,
			string(contract.ContractStates), pq.Array(dbtypes.NewDBCoins(contract.Funds)), contract.InstantiatedAt, contract.Height, contract.TxHash,
		)
	}

	// Remove trailing ","
	stmt = stmt[:len(stmt)-1]
	stmt += ` ON CONFLICT (contract_address) DO NOTHING`

	_, err := db.SQL.Exec(stmt, args...)
	if err != nil {
		return fmt.Errorf("failed to save WASM contracts: %s", err)
	}

	return nil
}

// GetWASMContractExists checks whether the specified WASM contract is currently stored inside the database.
func (db *DB) GetWASMContractExists(contractAddress string) (bool, error) {
	var count int
	err := db.SQL.Get(&count, `SELECT count(contract_address) FROM wasm_instantiate_contract WHERE contract_address = $1`, contractAddress)
	return count > 0, err
}

// SaveWASMExecuteContract allows to store one WASM execute contract.
func (db *DB) SaveWASMExecuteContract(wasmExecuteContract types.WASMExecuteContract) error {
	return db.SaveWASMExecuteContracts([]types.WASMExecuteContract{wasmExecuteContract})
}

// SaveWasmContracts allows to store WASM contract slice.
func (db *DB) SaveWASMExecuteContracts(executeContracts []types.WASMExecuteContract) error {
	paramsNumber := 9
	slices := dbutils.SplitWASMExecuteContracts(executeContracts, paramsNumber)

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

// SaveWASMExecuteContractEvents allows to store the WASM contract events.
func (db *DB) SaveWASMExecuteContractEvents(executeContract types.WASMExecuteContract, tx *junotypes.Transaction) error {
	stmt := `
	INSERT INTO wasm_execute_contract_event_types
	(contract_address, event_type, first_seen_height, first_seen_hash, last_seen_height, last_seen_hash)
	VALUES ($1, $2, $3, $4, $3, $4)
	ON CONFLICT (contract_address, event_type) DO UPDATE
	SET (last_seen_height, last_seen_hash) = (EXCLUDED.last_seen_height, EXCLUDED.last_seen_hash);
	`

	// Parse event if Cosmos SDK version is higher than 0.50
	// No need to keep compatible with old SDK version which event data is in tx.Logs
	for _, event := range tx.Events {
		for _, attr := range event.Attributes {
			if attr.Key == "msg_index" {
				_, err := db.SQL.Exec(stmt, executeContract.ContractAddress, event.Type, executeContract.Height, tx.TxHash)
				if err != nil {
					return fmt.Errorf("failed to save WASM execute contracts events: %s", err)
				}
			}
		}
	}

	return nil
}

func (db *DB) SaveWASMMigrateContracts(migrateContract types.WASMMigrateContract) error {
	stmt := `
INSERT INTO wasm_migrate_contract 
(sender, code_id, contract_address, migrate_contract_message, wasm_event, custom_wasm_event, migrated_at, height, tx_hash) 
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (contract_address, code_id) DO NOTHING`

	_, err := db.SQL.Exec(stmt,
		migrateContract.Sender, migrateContract.CodeID, migrateContract.ContractAddress, migrateContract.MigrateContractMsg,
		migrateContract.WASMEvent, migrateContract.CustomWASMEvent, migrateContract.MigratedAt, migrateContract.Height, migrateContract.TxHash,
	)
	if err != nil {
		return fmt.Errorf("failed to save WASM contract from contract migration: %s", err)
	}
	return nil
}

func (db *DB) UpdateContractAdmin(sender string, contractAddress string, newAdmin string) error {
	stmt := `UPDATE wasm_instantiate_contract SET sender = $1, admin = $2 WHERE contract_address = $2 `

	_, err := db.SQL.Exec(stmt, sender, newAdmin, contractAddress)
	if err != nil {
		return fmt.Errorf("failed to update WASM contract admin: %s", err)
	}
	return nil
}
