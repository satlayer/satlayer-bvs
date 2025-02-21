use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("StrategyBaseTvlLimits: unauthorized")]
    Unauthorized {},

    #[error("StrategyBaseTvlLimits.deposit: new_shares cannot be zero")]
    ZeroNewShares {},

    #[error("StrategyBaseTvlLimits.withdraw: amount to send cannot be zero")]
    ZeroAmountToSend {},

    #[error("StrategyBaseTvlLimits.withdraw: insufficient shares")]
    InsufficientShares {},

    #[error("StrategyBaseTvlLimits: invalid token")]
    InvalidToken {},

    #[error("StrategyBaseTvlLimits.withdraw: insufficient balance")]
    InsufficientBalance {},

    #[error("StrategyBaseTvlLimits.set_tvl_limits: invalid TVL limits")]
    InvalidTvlLimits {},

    #[error("StrategyBaseTvlLimits.deposit: max per deposit exceeded")]
    MaxPerDepositExceeded {},

    #[error("StrategyBaseTvlLimits.deposit: max total deposits exceeded")]
    MaxTotalDepositsExceeded {},
}
