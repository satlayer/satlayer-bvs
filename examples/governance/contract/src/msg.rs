use cosmwasm_schema::{cw_serde, QueryResponses};
use cw3_fixed_multisig;

/// Instantiate message for the contract.
#[cw_serde]
pub struct InstantiateMsg {
    pub registry: String,
    pub router: String,
    /// Used for administrative operations.
    pub owner: String,
    pub cw3_instantiate_msg: cw3_fixed_multisig::msg::InstantiateMsg,
}

#[cw_serde]
pub enum ExecuteMsg {
    Base(cw3_fixed_multisig::msg::ExecuteMsg),
    Extended(ExtendedExecuteMsg),
}

#[cw_serde]
pub enum ExtendedExecuteMsg {
    // Implement bvs specific custom extended execute messages here
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(cw3_fixed_multisig::msg::QueryMsg)]
    Base(cw3_fixed_multisig::msg::QueryMsg),
    #[returns(ExtendedQueryMsg)]
    Extended(ExtendedQueryMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum ExtendedQueryMsg {
    // Implement bvs specific custom extended execute messages here
}

#[cw_serde]
pub struct ServiceInfoResponse {
    pub owner: String,
    pub registry: String,
    pub router: String,
    pub slashing_enabled: bool,
}
