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
pub enum CanExecuteFlag {
    CanExecute = 0,
    IsPaused = 1,
    Unauthorized = 2,
}

#[cw_serde]
pub struct CanExecuteResponse(pub CanExecuteFlag);

impl CanExecuteResponse {
    pub fn new(flag: CanExecuteFlag) -> Self {
        Self(flag)
    }

    pub fn can_execute(&self) -> bool {
        self.0 == CanExecuteFlag::CanExecute
    }
}

#[cfg(test)]
mod tests {
    use super::{CanExecuteFlag, CanExecuteResponse, IsPausedResponse};

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
        let msg = CanExecuteResponse::new(CanExecuteFlag::CanExecute);
        assert_eq!(msg.can_execute(), true);

        let msg = CanExecuteResponse::new(CanExecuteFlag::IsPaused);
        assert_eq!(msg.can_execute(), false);

        let msg = CanExecuteResponse::new(CanExecuteFlag::Unauthorized);
        assert_eq!(msg.can_execute(), false);
    }
}
