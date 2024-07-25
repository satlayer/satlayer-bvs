use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("StrategyBase: unauthorized")]
    Unauthorized {},

    #[error("StrategyBase.deposit: new_shares cannot be zero")]
    ZeroNewShares {},

    #[error("StrategyBase.withdraw: insufficient shares")]
    InsufficientShares {},

    #[error("StrategyBase._after_withdrawal: amount overflow")]
    AmountOverflow {},
}
