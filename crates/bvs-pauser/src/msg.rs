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
        /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
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

#[cw_serde]
pub struct CanExecuteResponse(pub u32);

#[derive(Debug, PartialEq)]
pub enum CanExecuteFlag {
    CanExecute = 0,
    Paused = 1,
    Unauthorized = 2,
}

impl From<CanExecuteFlag> for CanExecuteResponse {
    fn from(flag: CanExecuteFlag) -> Self {
        Self(flag as u32)
    }
}

impl From<CanExecuteResponse> for CanExecuteFlag {
    fn from(value: CanExecuteResponse) -> Self {
        match value.0 {
            0 => Self::CanExecute,
            1 => Self::Paused,
            2 => Self::Unauthorized,
            _ => panic!("Unknown flag in CanExecuteResponse"),
        }
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
        let msg = CanExecuteResponse(0);
        assert_eq!(msg.0, 0);
        let flag: CanExecuteFlag = msg.into();
        assert_eq!(flag, CanExecuteFlag::CanExecute);

        let msg = CanExecuteResponse(1);
        assert_eq!(msg.0, 1);
        let flag: CanExecuteFlag = msg.into();
        assert_eq!(flag, CanExecuteFlag::Paused);

        let msg = CanExecuteResponse(2);
        assert_eq!(msg.0, 2);
        let flag: CanExecuteFlag = msg.into();
        assert_eq!(flag, CanExecuteFlag::Unauthorized);
    }
}
