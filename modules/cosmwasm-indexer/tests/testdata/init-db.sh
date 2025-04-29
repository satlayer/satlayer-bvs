#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
	CREATE USER user;
	CREATE DATABASE indexer;
	GRANT ALL PRIVILEGES ON DATABASE indexer TO user;
    CREATE TABLE wasm_execute_contract
    (
        sender                      TEXT            NOT NULL,
        contract_address            TEXT            NOT NULL,
        execute_contract_message    JSONB           NOT NULL DEFAULT '{}'::JSONB,
        message_type                TEXT            NULL,
        wasm_event                  JSONB           NOT NULL DEFAULT '{}'::JSONB,
        custom_wasm_event           JSONB           NOT NULL DEFAULT '{}'::JSONB,
        executed_at                 TIMESTAMP       NOT NULL,
        height                      BIGINT          NOT NULL,
        tx_hash                     TEXT            NOT NULL,
        PRIMARY KEY (height, tx_hash)
    );
EOSQL
