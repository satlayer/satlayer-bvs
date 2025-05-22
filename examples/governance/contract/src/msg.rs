use bvs_registry::SlashingParameters;
use cosmwasm_schema::cw_serde;
use cw3_fixed_multisig;

/// Instantiate message for the contract.
#[cw_serde]
pub struct InstantiateMsg {
    pub registry: String,
    pub router: String,
    /// Used for administrative operations.
    pub owner: String,
    pub cw3_instantiate_msg: cw3_fixed_multisig::msg::InstantiateMsg,
    pub initial_slashing_parameters: SlashingParameters,
    pub initial_member_list: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Base(cw3_fixed_multisig::msg::ExecuteMsg),
    Extended(ExtendedExecuteMsg),
}

#[cw_serde]
pub enum ExtendedExecuteMsg {}

#[cw_serde]
pub enum QueryMsg {
    Base(cw3_fixed_multisig::msg::QueryMsg),
    Extended(ExtendedQueryMsg),
}

#[cw_serde]
pub enum ExtendedQueryMsg {
    /// Returns the address of the router contract.
    GetRouter {},
    /// Returns the address of the registry contract.
    GetRegistry {},
    /// Returns the address of the owner of this contract.
    GetOwner {},
}
