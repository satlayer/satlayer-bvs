use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
    pub pauser: String,
}

#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum ExecuteMsg {
    /// ExecuteMsg SetVault the vault contract in the router and whitelist (true/false) it.
    /// Only the `owner` can call this message.
    SetVault { vault: String, whitelisted: bool },

    /// ExecuteMsg TransferOwnership
    /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
    TransferOwnership { new_owner: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// QueryMsg IsWhitelisted: returns true if the vault is whitelisted.
    /// See [`ExecuteMsg::SetVault`]
    #[returns(IsWhitelistedResponse)]
    IsWhitelisted { vault: String },

    /// QueryMsg IsValidating: returns true if the operator is validating services.
    /// See BVS Registry for more information.
    #[returns(IsValidatingResponse)]
    IsValidating { operator: String },

    /// QueryMsg ListVaults: returns a list of vaults.
    /// You can provide `limit` and `start_after` to paginate the results.
    /// The max `limit` is 100.
    #[returns(VaultListResponse)]
    ListVaults {
        limit: Option<u32>,
        start_after: Option<String>,
    },
}

/// The response to the `IsWhitelisted` query.
/// Not exported.
/// This is just a wrapper around `bool`, so that the schema can be generated.
#[cw_serde]
struct IsWhitelistedResponse(bool);

/// The response to the `IsValidating` query.
/// Not exported.
/// This is just a wrapper around `bool`, so that the schema can be generated.
#[cw_serde]
struct IsValidatingResponse(bool);

/// The response to the `ListVaults` query.
/// For pagination, the `start_after` field is the last `vault` from the previous page.
#[cw_serde]
pub struct VaultListResponse(pub Vec<Vault>);

#[cw_serde]
pub struct Vault {
    pub vault: Addr,
    pub whitelisted: bool,
}
