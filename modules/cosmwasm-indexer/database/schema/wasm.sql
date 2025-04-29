CREATE TYPE ACCESS_CONFIG AS
(
    permission  INT,
    address     TEXT
);

CREATE TYPE COIN AS
(
    denom  TEXT,
    amount TEXT
);

CREATE TABLE wasm_params
(
    one_row_id                      BOOLEAN         NOT NULL DEFAULT TRUE PRIMARY KEY,
    code_upload_access              ACCESS_CONFIG   NOT NULL,
    instantiate_default_permission  INT             NOT NULL,
    height                          BIGINT          NOT NULL
);

CREATE TABLE wasm_code
(
    sender                  TEXT            NULL,
    byte_code               BYTEA           NOT NULL,
    instantiate_permission  ACCESS_CONFIG   NULL,
    code_id                 BIGINT          NOT NULL UNIQUE,
    height                  BIGINT          NOT NULL
);
CREATE INDEX wasm_code_height_index ON wasm_code (height);

CREATE TABLE wasm_instantiate_contract
(
    sender                          TEXT            NULL,
    creator                         TEXT            NOT NULL,
    admin                           TEXT            NULL,
    code_id                         BIGINT          NOT NULL REFERENCES wasm_code (code_id),
    label                           TEXT            NULL,
    instantiate_contract_message    JSONB           NOT NULL DEFAULT '{}'::JSONB,
    contract_address                TEXT            NOT NULL UNIQUE,
    wasm_event                      JSONB           NOT NULL DEFAULT '{}'::JSONB,
    custom_wasm_event               JSONB           NOT NULL DEFAULT '{}'::JSONB,
    contract_info_extension         TEXT            NULL,
    contract_states                 JSONB           NOT NULL DEFAULT '{}'::JSONB,
    funds                           COIN[]          NOT NULL DEFAULT '{}',
    instantiated_at                 TIMESTAMP       NOT NULL,
    height                          BIGINT          NOT NULL,
    tx_hash                         TEXT            NOT NULL
);
CREATE INDEX wasm_instantiate_contract_height_index ON wasm_instantiate_contract (height);
CREATE INDEX wasm_instantiate_contract_creator_index ON wasm_instantiate_contract (creator);
CREATE INDEX wasm_instantiate_contract_label_index ON wasm_instantiate_contract (label);

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
CREATE INDEX execute_contract_address_index ON wasm_execute_contract (contract_address);
CREATE INDEX execute_contract_executed_at_index ON wasm_execute_contract (executed_at);
CREATE INDEX execute_contract_message_type_index ON wasm_execute_contract (message_type);

CREATE TABLE wasm_execute_contract_event_types
(
    contract_address         TEXT   NOT NULL REFERENCES wasm_instantiate_contract (contract_address),
    event_type               TEXT   NOT NULL,

    first_seen_height        BIGINT NOT NULL REFERENCES block (height),
    first_seen_hash          TEXT   NOT NULL,

    last_seen_height         BIGINT NOT NULL REFERENCES block (height),
    last_seen_hash           TEXT   NOT NULL,
    UNIQUE (contract_address, event_type)
);
CREATE INDEX wasm_execute_contract_event_types_index ON wasm_execute_contract_event_types (contract_address, event_type);

CREATE TABLE wasm_migrate_contract
(
    sender                      TEXT            NULL,
    code_id                     BIGINT          NOT NULL REFERENCES wasm_code (code_id),
    contract_address            TEXT            NOT NULL REFERENCES wasm_instantiate_contract (contract_address),
    migrate_contract_message    JSONB           NOT NULL DEFAULT '{}'::JSONB,
    wasm_event                  JSONB           NOT NULL DEFAULT '{}'::JSONB,
    custom_wasm_event           JSONB           NOT NULL DEFAULT '{}'::JSONB,
    migrated_at                 TIMESTAMP       NOT NULL,
    height                      BIGINT          NOT NULL,
    tx_hash                     TEXT            NOT NULL,
    PRIMARY KEY (contract_address, code_id)
);
CREATE INDEX wasm_migrate_contract_code_id_index ON wasm_migrate_contract (code_id);
CREATE INDEX wasm_migrate_contract_contract_address_index ON wasm_migrate_contract (contract_address);
