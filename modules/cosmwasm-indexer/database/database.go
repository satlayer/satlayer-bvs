package database

import (
	"fmt"

	"github.com/forbole/juno/v6/database"
	"github.com/forbole/juno/v6/database/postgresql"
	junotypes "github.com/forbole/juno/v6/types"
	"github.com/jmoiron/sqlx"
	indexertypes "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types"
)

// DB represents a PostgreSQL database with expanded features.
// so that it can properly store posts and other Wasm-related data.
type DB struct {
	*postgresql.Database
	sqlxDB *sqlx.DB
}

// Cast casts the given database to be a *DB
func Cast(database database.Database) *DB {
	wasmDB, ok := (database).(*DB)
	if !ok {
		panic(fmt.Errorf("database is not a WasmDB instance"))
	}
	return wasmDB
}

// Builder allows to create a new Db instance implementing the database.Builder type
func Builder(ctx *database.Context) (database.Database, error) {
	db, err := postgresql.Builder(ctx)
	if err != nil {
		return nil, err
	}

	psqlDB, ok := (db).(*postgresql.Database)
	if !ok {
		return nil, fmt.Errorf("invalid database type")
	}

	return &DB{
		Database: psqlDB,
		sqlxDB:   sqlx.NewDb(psqlDB.SQL.DB, "postgresql"),
	}, nil
}

// HasBlock overrides postgresql.Database to perform a no-op
func (db *DB) HasBlock(height int64) (bool, error) {
	return db.Database.HasBlock(height)
}

// GetLastBlockHeight overrides postgresql.Database to perform a no-op
func (db *DB) GetLastBlockHeight() (int64, error) {
	return db.Database.GetLastBlockHeight()
}

// GetMissingHeights overrides postgresql.Database to perform a no-op
func (db *DB) GetMissingHeights(startHeight, endHeight int64) []int64 {
	return db.Database.GetMissingHeights(startHeight, endHeight)
}

// SaveBlock overrides postgresql.Database to perform a no-op
func (db *DB) SaveBlock(block *junotypes.Block) error {
	return db.Database.SaveBlock(block)
}

// GetTotalBlocks overrides postgresql.Database to perform a no-op
func (db *DB) GetTotalBlocks() int64 {
	return db.Database.GetTotalBlocks()
}

// SaveTx overrides postgresql.Database to perform a no-op
func (db *DB) SaveTx(tx *junotypes.Transaction) error {
	return db.Database.SaveTx(tx)
}

// HasValidator overrides postgresql.Database to perform a no-op
func (db *DB) HasValidator(addr string) (bool, error) {
	return db.Database.HasValidator(addr)
}

// SaveValidators overrides postgresql.Database to perform a no-op
func (db *DB) SaveValidators(validators []*junotypes.Validator) error {
	return db.Database.SaveValidators(validators)
}

// SaveCommitSignatures overrides postgresql.Database to perform a no-op
func (db *DB) SaveCommitSignatures(signatures []*junotypes.CommitSig) error {
	return db.Database.SaveCommitSignatures(signatures)
}

// SaveMessage overrides postgresql.Database to perform a no-op
func (db *DB) SaveMessage(height int64, txHash string, msg junotypes.Message, addresses []string) error {
	return db.Database.SaveMessage(height, txHash, msg, addresses)
}

// SaveCode saves a code to the database
func (db *DB) SaveCode(code *indexertypes.Code) error {
	return nil
}

// SaveContract saves a contract to the database
func (db *DB) SaveContract(contract *indexertypes.Contract) error {
	return nil
}

// SaveContractCodeID saves a contract code ID to the database
func (db *DB) SaveContractCodeID(contract string, codeID uint64) error {
	return nil
}

// UpdateContractAdmin updates a contract admin in the database
func (db *DB) UpdateContractAdmin(contract string, admin string) error {
	return nil
}
