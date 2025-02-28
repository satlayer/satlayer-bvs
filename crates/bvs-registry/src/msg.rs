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

    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(IsPausedResponse)]
    IsPaused {
        /// The (contract: Addr) calling this
        #[serde(rename = "c")]
        contract: String,
        /// The (method: ExecuteMsg) to check if it is paused
        #[serde(rename = "m")]
        method: String,
    },

    #[returns(CanExecuteResponse)]
    CanExecute {
        /// The (contract: Addr) calling this
        #[serde(rename = "c")]
        contract: String,
        /// The (sender: Addr) of the message
        #[serde(rename = "s")]
        sender: String,
        /// The (method: ExecuteMsg) to check if it is paused
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

pub const FLAG_CAN_EXECUTE: u32 = 0;
pub const FLAG_PAUSED: u32 = 1;
pub const FLAG_UNAUTHORIZED: u32 = 2;

#[cw_serde]
pub struct CanExecuteResponse(pub u32);

impl CanExecuteResponse {
    pub fn new(flag: u32) -> Self {
        Self(flag)
    }

    pub fn can_execute(&self) -> bool {
        self.0 == FLAG_CAN_EXECUTE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paused() {
        let msg = IsPausedResponse::new(true);
        assert_eq!(msg.is_paused(), true);

        let msg = IsPausedResponse::new(false);
        assert_eq!(msg.is_paused(), false);
    }

    /// Test the raw value of the IsPausedResponse
    /// If != 1, it is not paused
    #[test]
    fn test_paused_raw() {
        let msg = IsPausedResponse(0);
        assert_eq!(msg.is_paused(), false);

        let msg = IsPausedResponse(1);
        assert_eq!(msg.is_paused(), true);
    }

    #[test]
    fn test_can_execute() {
        let msg = CanExecuteResponse::new(FLAG_CAN_EXECUTE);
        assert_eq!(msg.can_execute(), true);
        assert_eq!(msg.0, 0);

        let msg = CanExecuteResponse::new(FLAG_PAUSED);
        assert_eq!(msg.can_execute(), false);
        assert_eq!(msg.0, 1);

        let msg = CanExecuteResponse::new(FLAG_UNAUTHORIZED);
        assert_eq!(msg.can_execute(), false);
        assert_eq!(msg.0, 2);
    }

    #[test]
    fn test_can_execute_raw() {
        let msg = CanExecuteResponse(FLAG_CAN_EXECUTE);
        assert_eq!(msg.can_execute(), true);
        assert_eq!(msg.0, 0);

        let msg = CanExecuteResponse(FLAG_PAUSED);
        assert_eq!(msg.can_execute(), false);
        assert_eq!(msg.0, 1);

        let msg = CanExecuteResponse(FLAG_UNAUTHORIZED);
        assert_eq!(msg.can_execute(), false);
        assert_eq!(msg.0, 2);
    }
}
