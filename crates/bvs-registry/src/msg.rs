use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner of this contract, who can pause and unpause
    pub owner: String,
    /// Initial pause state
    pub initial_paused: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    Pause {},
    Unpause {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(IsPausedResponse)]
    IsPaused {
        /// The address of the caller (contract)
        sender: String,
        /// The ExecuteMsg method to check if it is paused
        method: String,
    },
}

#[cw_serde]
pub struct IsPausedResponse {
    pub paused: bool,
}
