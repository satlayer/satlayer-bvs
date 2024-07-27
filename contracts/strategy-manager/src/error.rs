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

    #[error("StrategyManager.deposit_into_strategy_with_signature: signature expired")]
    SignatureExpired {},

    #[error("StrategyManager.deposit_into_strategy_with_signature: invalid signature")]
    InvalidSignature {},

    #[error("StrategyManager.deposit_into_strategy_with_signature: third transfers disabled")]
    ThirdTransfersDisabled {},
}
