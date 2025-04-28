package database

import (
	"fmt"
	"log"

	"github.com/forbole/juno/v6/database"
	"github.com/forbole/juno/v6/database/postgresql"
	"github.com/jmoiron/sqlx"
)

// DB represents a PostgreSQL database with expanded features.
// so that it can properly store Wasm-related data.
type DB struct {
	*postgresql.Database
	sqlxDB *sqlx.DB
}

// Cast casts the given database to be a *DB
func Cast(database database.Database) *DB {
	db, ok := (database).(*DB)
	if !ok {
		log.Fatal("cannot cast the given db into an instance")
	}
	return db
}

// Builder allows to create a new DB instance implementing the database.Builder type
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
