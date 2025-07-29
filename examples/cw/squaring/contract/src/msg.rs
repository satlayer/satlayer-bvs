use cosmwasm_schema::{cw_serde, QueryResponses};

/// Instantiate message for the contract.
#[cw_serde]
pub struct InstantiateMsg {
    pub registry: String,
    pub router: String,
    /// Used for administrative operations.
    pub owner: String,
}

/// Migration message for the contract.
/// > This is used when the contract is migrated to a new version.
/// > You can keep this empty if you don't need to do anything special during migration.
#[cw_serde]
pub struct MigrateMsg {}

/// Execute messages for the contract.
/// These messages allow modifying the state of the contract and emits event.
#[cw_serde]
pub enum ExecuteMsg {
    /// Request for a new `input` to be computed.
    Request { input: i64 },
    /// Respond to a `Request` with the computed `output`.
    Respond { input: i64, output: i64 },
    /// Compute the square of the `input` on-chain to correct the `output`.
    /// The operator that responded to the request with the wrong output will be slashed.
    /// This will be used to kick-start the slashing process.
    /// If the operator can't be slashed,
    /// the contract will still apply but the operator will not be slashed to allow of service continuity.
    Compute { input: i64, operator: String },
    /// Resolve the slashing process for the `operator` by canceling it.
    SlashingCancel { operator: String },
    /// Move to the 2nd stage of the slashing process, locking the operator vault's funds.
    SlashingLock { operator: String },
    /// Finalize the slashing process for the `operator`.
    SlashingFinalize { operator: String },
    /// Register a new operator for the squaring service.
    RegisterOperator { operator: String },
    /// Deregister an operator from running the squaring service.
    DeregisterOperator { operator: String },
    /// Enable slashing for the squaring service.
    EnableSlashing {},
    /// Disable slashing for the squaring service.
    DisableSlashing {},
}

/// Query messages for the contract.
/// These messages allow querying the state of the contract.
/// Does not allow modifying the state of the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get response for a given `input` with `operator` that responded to the request.
    #[returns(i64)]
    GetResponse { input: i64, operator: String },
}
