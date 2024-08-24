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

    #[error("StrategyManager.add_strategies_to_whitelist: Strategies and third party transfers forbidden values lengths do not match")]
    InvalidInput {},

    #[error("StrategyManager.remove_strategy_from_staker_strategy_list: strategy not found")]
    StrategyNotFound {},

    #[error("StrategyManager._deposit_into_strategy: zero new shares")]
    ZeroNewShares {},

    #[error("StrategyManager.deposit_into_strategy_with_signature: attribute not found")]
    AttributeNotFound {},

    #[error("StrategyManager.calculate_new_shares: Overflow occurred during calculation")]
    Overflow,

    #[error("StrategyManager.calculate_new_shares: Underflow occurred during calculation")]
    Underflow,

    #[error("StrategyManager.calculate_new_shares: Division by zero")]
    DivideByZero,
}
