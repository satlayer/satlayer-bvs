use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
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
    #[returns(IsWhitelistedResponse)]
    IsWhitelisted { vault: String },

    #[returns(IsValidatingResponse)]
    IsValidating { operator: String },

    #[returns(VaultListResponse)]
    ListVaults {
        limit: Option<u32>,
        start_after: Option<String>,
    },
}

#[cw_serde]
struct IsWhitelistedResponse(bool);

#[cw_serde]
struct IsValidatingResponse(bool);

#[cw_serde]
pub struct VaultListResponse(pub Vec<Vault>);

#[cw_serde]
pub struct Vault {
    pub vault: Addr,
    pub whitelisted: bool,
}
