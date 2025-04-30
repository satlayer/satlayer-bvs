use cosmwasm_schema::{cw_serde, QueryResponses};

/// Instantiate message for the contract.
#[cw_serde]
pub struct InstantiateMsg {}

/// Migration message for the contract.
/// > This is used when the contract is migrated to a new version.
/// > You can keep this empty if you don't need to do anything special during migration.
#[cw_serde]
pub struct MigrateMsg {}

/// Execute messages for the contract.
/// These messages allow modifying the state of the contract and emits event.
#[cw_serde]
pub enum ExecuteMsg {
    /// ExecuteMsg Request for a new `input` to be computed.
    Request { input: i64 },
    /// ExecuteMsg Respond to a `Request` with the computed `output`.
    Respond { input: i64, output: i64 },
    /// ExecuteMsg Prove by computing the square of the `input` on-chain to correct the `output`.
    /// The operator that responded to the request with the wrong output will be slashed.
    Prove { input: i64, operator: String },
}

/// Query messages for the contract.
/// These messages allow querying the state of the contract.
/// Does not allow modifying the state of the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// QueryMsg GetResponse for a given `input` with `operator` that responded to the request.
    #[returns(i64)]
    GetResponse { input: i64, operator: String },
}
