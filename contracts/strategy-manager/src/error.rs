use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("StrategyManager: strategy not whitelisted")]
    StrategyNotWhitelisted {},

    #[error("StrategyManager: unauthorized")]
    Unauthorized {},

    #[error("StrategyManager.add_shares: invalid shares")]
    InvalidShares {},

    #[error("StrategyManager.add_shares: max strategy list length exceeded")]
    MaxStrategyListLengthExceeded {},
}
