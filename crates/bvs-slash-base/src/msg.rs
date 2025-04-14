use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

/// Slash `ExecuteMsg`, to be implemented by the Slash strategy specific contract.
#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum SlasherExecuteMsg<Offense, Evidence, SlashDetail> {
    /// SubmitOffense: submit an offense to the slash strategy contract.
    /// offender: the address of the operator.
    /// offense: the offense committed. (Opaque to the slasher contract)
    /// evidence: the evidence of the offense. (Opaque to the slasher contract)
    SubmitSlash(SubmitSlash<Offense, Evidence>),

    /// ExecuteSlash: trigger the slash on slash strategy contract to execute the slash.
    /// Exacatly how the slash is executed is up to the slash strategy contract.
    /// How the slash entries will be hash is also up to the slash strategy contract.
    /// slash_detail: the hash of the slash.
    ExecuteSlash(ExecuteSlash<SlashDetail>),
}

#[cw_serde]
pub struct SubmitSlash<Offense, Evidence> {
    /// The operator who is being accused of misbehaviour
    pub offender: String,

    /// A generic type for downstream slashing strategy contract should define
    /// allowing them to have custom slashing strategy
    pub offense: Offense,
    pub evidence: Evidence,
}

#[cw_serde]
pub struct ExecuteSlash<SlashDetail> {
    pub slash_detail: SlashDetail,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum SlasherQueryMsg {
    /// QueryMsg VaultInfo: get the vault information.
    #[returns(SlasherInfoResponse)]
    SlasherInfo {},
}

#[cw_serde]
pub struct SlasherInfoResponse {
    /// Queued slash entries
    pub outstanding_slash: Uint128,

    /// The `vault-router` contract address
    pub router: Addr,

    /// The `pauser` contract address
    pub pauser: Addr,

    /// The `vault` contract address where this slasher contract is managing slash for
    pub vault: String,

    /// The name of the vault contract, see [`cw2::set_contract_version`] for more information.
    pub contract: String,

    /// The version of the vault contract, see [`cw2::set_contract_version`] for more information.
    pub version: String,
}
