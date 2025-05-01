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
