use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner of this contract, who can pause and unpause
    pub owner: String,
    /// The initial paused state of this contract
    pub initial_paused: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// ExecuteMsg Pause pauses a method on a contract.
    /// Callable by the owner of the pauser contract
    Pause {
        /// address of the contract to be paused
        contract: String,
        /// method of a particular contract to be paused
        method: String,
    },

    /// ExecuteMsg Unpause unpauses a method on a contract.
    /// Callable by the owner of the pauser contract
    Unpause {
        /// address of the contract to be unpaused
        contract: String,
        /// method of a particular contract to be unpaused
        method: String,
    },

    /// ExecuteMsg PauseGlobal pauses all execution on all contracts and methods.
    /// Callable by the owner of the pauser contract
    PauseGlobal {},

    /// ExecuteMsg UnpauseGlobal unpauses all execution.
    /// Callable by the owner of the pauser contract
    /// Unpauses Globally
    UnpauseGlobal {},

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
        assert!(msg.is_paused());

        let msg = IsPausedResponse::new(false);
        assert!(!msg.is_paused());
    }

    /// Test the raw value of the IsPausedResponse
    /// If != 1, it is not paused
    #[test]
    fn test_paused_raw() {
        let msg = IsPausedResponse(0);
        assert!(!msg.is_paused());

        let msg = IsPausedResponse(1);
        assert!(msg.is_paused());
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
