package database

import (
	"fmt"

	junotypes "github.com/forbole/juno/v6/types"
	"github.com/lib/pq"

	dbtypes "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database/types"
	dbutils "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database/utils"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"
)

// SaveWasmParams allows to store the wasm params
func (db *DB) SaveWasmParams(params types.WasmParams) error {
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
		return fmt.Errorf("error while saving wasm params: %s", err)
	}

	return nil
}

// SaveWasmCode allows to store a single wasm code
func (db *DB) SaveWasmCode(wasmCode types.WasmCode) error {
	return db.SaveWasmCodes([]types.WasmCode{wasmCode})
}

// SaveWasmCodes allows to store the wasm code slice
func (db *DB) SaveWasmCodes(wasmCodes []types.WasmCode) error {
	stmt := `
INSERT INTO wasm_code(sender, byte_code, instantiate_permission, code_id, height) 
VALUES `

	var args []interface{}
	for i, code := range wasmCodes {
		ii := i * 5

		var accessConfig dbtypes.DBAccessConfig
		if code.InstantiatePermission != nil {
			accessConfig = dbtypes.NewDBAccessConfig(code.InstantiatePermission)
		}

		cfgValue, _ := accessConfig.Value()

		stmt += fmt.Sprintf("($%d, $%d, $%d, $%d, $%d),", ii+1, ii+2, ii+3, ii+4, ii+5)
		args = append(args, code.Sender, code.WasmByteCode, cfgValue, code.CodeID, code.Height)
	}

	stmt = stmt[:len(stmt)-1] // Remove trailing ","

	stmt += `
	ON CONFLICT (code_id) DO UPDATE 
		SET sender = excluded.sender,
			byte_code = excluded.byte_code,
			instantiate_permission = excluded.instantiate_permission,
			height = excluded.height
	WHERE wasm_code.height <= excluded.height`

	_, err := db.SQL.Exec(stmt, args...)
	if err != nil {
		return fmt.Errorf("error while saving wasm code: %s", err)
	}

	return nil
}

// SaveWasmContracts allows to store the wasm contract slice
func (db *DB) SaveWasmContracts(contracts []types.WasmContract) error {
	paramsNumber := 13
	slices := dbutils.SplitWasmContracts(contracts, paramsNumber)

	for _, contracts := range slices {
		if len(contracts) == 0 {
			continue
		}

		err := db.saveWasmContracts(paramsNumber, contracts)
		if err != nil {
			return fmt.Errorf("error while storing contracts: %s", err)
		}
	}

	return nil
}

func (db *DB) saveWasmContracts(paramsNumber int, wasmContracts []types.WasmContract) error {
	stmt := `
INSERT INTO wasm_contract 
(sender, creator, admin, code_id, label, raw_contract_message, funds, contract_address, 
data, instantiated_at, contract_info_extension, contract_states, height) 
VALUES `

	var args []interface{}
	for i, contract := range wasmContracts {
		ii := i * paramsNumber
		stmt += fmt.Sprintf("($%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d),",
			ii+1, ii+2, ii+3, ii+4, ii+5, ii+6, ii+7, ii+8, ii+9, ii+10, ii+11, ii+12, ii+13)
		args = append(args,
			contract.Sender, contract.Creator, contract.Admin, contract.CodeID, contract.Label, string(contract.RawContractMsg),
			pq.Array(dbtypes.NewDBCoins(contract.Funds)), contract.ContractAddress, contract.Data,
			contract.InstantiatedAt, contract.ContractInfoExtension, string(contract.ContractStates), contract.Height,
		)
	}

	stmt = stmt[:len(stmt)-1] // Remove trailing ","
	stmt += `
	ON CONFLICT (contract_address) DO UPDATE 
		SET sender = excluded.sender,
			creator = excluded.creator,
			admin = excluded.admin,
			code_id = excluded.code_id,
			label = excluded.label,
			raw_contract_message = excluded.raw_contract_message,
			funds = excluded.funds,
			data = excluded.data,
			instantiated_at = excluded.instantiated_at,
			contract_info_extension = excluded.contract_info_extension,
			contract_states = excluded.contract_states,
			height = excluded.height
	WHERE wasm_contract.height <= excluded.height`

	_, err := db.SQL.Exec(stmt, args...)
	if err != nil {
		return fmt.Errorf("error while saving wasm contracts: %s", err)
	}

	return nil
}

// GetWasmContractExists returns all the wasm contracts matching an address that are currently stored inside the database.
func (db *DB) GetWasmContractExists(contractAddress string) (bool, error) {
	var count int
	err := db.SQL.Select(&count, `SELECT count(contract_address) FROM wasm_contract WHERE contract_address = '`+contractAddress+`'`)
	return count > 0, err
}

// SaveWasmExecuteContract allows to store the wasm contract
func (db *DB) SaveWasmExecuteContract(wasmExecuteContract types.WasmExecuteContract) error {
	return db.SaveWasmExecuteContracts([]types.WasmExecuteContract{wasmExecuteContract})
}

// SaveWasmContracts allows to store the wasm contract slice
func (db *DB) SaveWasmExecuteContracts(executeContracts []types.WasmExecuteContract) error {
	paramsNumber := 8
	slices := dbutils.SplitWasmExecuteContracts(executeContracts, paramsNumber)

	for _, contracts := range slices {
		if len(contracts) == 0 {
			continue
		}

		err := db.saveWasmExecuteContracts(paramsNumber, executeContracts)
		if err != nil {
			return fmt.Errorf("error while storing contracts: %s", err)
		}
	}

	return nil
}

func (db *DB) saveWasmExecuteContracts(paramNumber int, executeContracts []types.WasmExecuteContract) error {
	stmt := `
INSERT INTO wasm_execute_contract 
(sender, contract_address, raw_contract_message, funds, data, executed_at, height, hash, message_type) 
VALUES `

	var args []interface{}
	for i, executeContract := range executeContracts {
		ii := i * paramNumber
		stmt += fmt.Sprintf("($%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d, $%d),",
			ii+1, ii+2, ii+3, ii+4, ii+5, ii+6, ii+7, ii+8, ii+9)
		args = append(args,
			executeContract.Sender, executeContract.ContractAddress, string(executeContract.RawContractMsg),
			pq.Array(dbtypes.NewDBCoins(executeContract.Funds)), executeContract.Data, executeContract.ExecutedAt, executeContract.Height, executeContract.Hash, executeContract.MessageType)
	}

	stmt = stmt[:len(stmt)-1] // Remove trailing ","

	stmt += ` ON CONFLICT DO NOTHING`

	_, err := db.SQL.Exec(stmt, args...)
	if err != nil {
		return fmt.Errorf("error while saving wasm execute contracts: %s", err)
	}

	return nil
}

// SaveWasmExecuteContractEvents allows to store the wasm contract events
func (db *DB) SaveWasmExecuteContractEvents(executeContract types.WasmExecuteContract, tx *junotypes.Transaction) error {
	stmt := `
	INSERT INTO wasm_execute_contract_event_types
	(contract_address,
	event_type,
	first_seen_height,
	first_seen_hash,
	last_seen_height,
	last_seen_hash)
	VALUES ($1, $2, $3, $4, $3, $4)
	ON CONFLICT (contract_address, event_type) DO UPDATE
	SET (last_seen_height, last_seen_hash) = (EXCLUDED.last_seen_height, EXCLUDED.last_seen_hash);
	`
	// If the logs are present, we are using pre-0.50
	// Log parsing is still needed because events don't have a msg_index SDK <0.50
	// and ignoring that will index a lot of unwanted values and bloat DB
	if len(tx.Logs) > 0 {
		for _, txLog := range tx.Logs {
			for _, event := range txLog.Events {

				_, err := db.SQL.Exec(stmt, executeContract.ContractAddress, event.Type, executeContract.Height, tx.TxHash)
				if err != nil {
					return fmt.Errorf("error while saving wasm execute contracts: %s", err)
				}
			}
		}
	} else {
		// We fall back to events for the newer version of SDK and look for events with msg_index set
		for _, event := range tx.Events {
			for _, attr := range event.Attributes {
				if attr.Key == "msg_index" {
					_, err := db.SQL.Exec(stmt, executeContract.ContractAddress, event.Type, executeContract.Height, tx.TxHash)
					if err != nil {
						return fmt.Errorf("error while saving wasm execute contracts: %s", err)
					}
				}
			}
		}
	}

	return nil
}

func (db *DB) UpdateContractWithMsgMigrateContract(
	sender string, contractAddress string, codeID uint64, rawContractMsg []byte, data string,
) error {
	stmt := `UPDATE wasm_contract SET 
sender = $1, code_id = $2, raw_contract_message = $3, data = $4 
WHERE contract_address = $5 `

	_, err := db.SQL.Exec(stmt,
		sender, codeID, string(rawContractMsg), data,
		contractAddress,
	)
	if err != nil {
		return fmt.Errorf("error while updating wasm contract from contract migration: %s", err)
	}
	return nil
}

func (db *DB) UpdateContractAdmin(sender string, contractAddress string, newAdmin string) error {
	stmt := `UPDATE wasm_contract SET 
sender = $1, admin = $2 WHERE contract_address = $2 `

	_, err := db.SQL.Exec(stmt, sender, newAdmin, contractAddress)
	if err != nil {
		return fmt.Errorf("error while updating wsm contract admin: %s", err)
	}
	return nil
}
