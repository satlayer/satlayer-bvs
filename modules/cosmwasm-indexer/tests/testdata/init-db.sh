#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
	GRANT ALL PRIVILEGES ON DATABASE indexer TO "docker";
	CREATE TABLE validator
  (
      consensus_address TEXT NOT NULL PRIMARY KEY, /* Validator consensus address */
      consensus_pubkey  TEXT NOT NULL UNIQUE /* Validator consensus public key */
  );
	CREATE TABLE block
  (
      height           BIGINT UNIQUE PRIMARY KEY,
      hash             TEXT                        NOT NULL UNIQUE,
      num_txs          INTEGER DEFAULT 0,
      total_gas        BIGINT  DEFAULT 0,
      proposer_address TEXT REFERENCES validator (consensus_address),
      timestamp        TIMESTAMP WITHOUT TIME ZONE NOT NULL
  );
  CREATE TABLE pre_commit
  (
      validator_address TEXT                        NOT NULL REFERENCES validator (consensus_address),
      height            BIGINT                      NOT NULL,
      timestamp         TIMESTAMP WITHOUT TIME ZONE NOT NULL,
      voting_power      BIGINT                      NOT NULL,
      proposer_priority BIGINT                      NOT NULL,
      UNIQUE (validator_address, timestamp)
  );
  CREATE TABLE transaction
  (
      hash         TEXT    NOT NULL,
      height       BIGINT  NOT NULL REFERENCES block (height),
      success      BOOLEAN NOT NULL,

      /* Body */
      messages     JSON    NOT NULL DEFAULT '[]'::JSON,
      memo         TEXT,
      signatures   TEXT[]  NOT NULL,

      /* AuthInfo */
      signer_infos JSONB   NOT NULL DEFAULT '[]'::JSONB,
      fee          JSONB   NOT NULL DEFAULT '{}'::JSONB,

      /* Tx response */
      gas_wanted   BIGINT           DEFAULT 0,
      gas_used     BIGINT           DEFAULT 0,
      raw_log      TEXT,
      logs         JSONB,

      /* PSQL partition */
      partition_id BIGINT  NOT NULL DEFAULT 0,

      CONSTRAINT unique_tx UNIQUE (hash, partition_id)
  ) PARTITION BY LIST (partition_id);
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
