use bvs_pauser::api::Display;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub pauser: String,
    pub registry: String,
    pub router: String,
}

#[cw_serde]
#[derive(Display)]
pub enum ExecuteMsg {
    /// ExecuteMsg DeployCw20
    /// Deploy a CW20 vault contract, the operator will be the sender of this message.
    /// The `cw20` is the address of the CW20 contract.
    DeployCw20 { cw20: String },

    /// ExecuteMsg DeployBank
    /// Deploy a Bank vault contract, the operator will be the sender of this message.
    /// The `denom` is the denomination of the native token, e.g. "ubbn" for Babylon native token.
    DeployBank { denom: String },

    /// ExecuteMsg TransferOwnership
    /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
    /// Only the `owner` can call this message.
    TransferOwnership { new_owner: String },

    /// ExecuteMsg SetCodeId
    /// Set the code id for a vault type, allowing the factory to deploy vaults of that type.
    /// Only the `owner` can call this message.
    SetCodeId { code_id: u64, vault_type: VaultType },
}

#[cw_serde]
#[derive(Display)]
pub enum VaultType {
    Bank,
    Cw20,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CodeIdResponse)]
    CodeId { vault_type: VaultType },
}

/// The response to the `CodeId` query.
/// Not exported.
/// This is just a wrapper around `u64`, so that the schema can be generated.
#[cw_serde]
struct CodeIdResponse(u64);
