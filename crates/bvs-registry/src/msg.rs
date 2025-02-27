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
        /// The contract calling this
        #[serde(rename = "c")]
        contract: String,
        /// The ExecuteMsg method to check if it is paused
        #[serde(rename = "m")]
        method: String,
    },

    #[returns(CanExecuteResponse)]
    CanExecute {
        /// The contract calling this
        #[serde(rename = "c")]
        contract: String,
        /// The sender of the message
        #[serde(rename = "s")]
        sender: String,
        /// The ExecuteMsg method to check if it is paused
        #[serde(rename = "m")]
        method: String,
    },
}

#[cw_serde]
pub struct IsPausedResponse(pub u32);

impl IsPausedResponse {
    pub fn new(paused: bool) -> Self {
        Self(paused as u32)
    }

    pub fn is_paused(&self) -> bool {
        self.0 == 1
    }
}

#[cw_serde]
pub struct CanExecuteResponse(pub u32);

impl CanExecuteResponse {
    pub fn new(can_execute: bool) -> Self {
        Self(can_execute as u32)
    }

    pub fn can_execute(&self) -> bool {
        self.0 == 1
    }
}
