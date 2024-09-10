use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("StrategyBaseTVLLimits: unauthorized")]
    Unauthorized {},

    #[error("StrategyBaseTVLLimits.deposit: new_shares cannot be zero")]
    ZeroNewShares {},

    #[error("StrategyBaseTVLLimits.withdraw: amount to send cannot be zero")]
    ZeroAmountToSend {},

    #[error("StrategyBaseTVLLimits.withdraw: insufficient shares")]
    InsufficientShares {},

    #[error("StrategyBaseTVLLimits: invalid token")]
    InvalidToken {},

    #[error("StrategyBaseTVLLimits.withdraw: insufficient balance")]
    InsufficientBalance {},

    #[error("StrategyBaseTVLLimits.set_tvl_limits: invalid TVL limits")]
    InvalidTVLLimits {},

    #[error("StrategyBaseTVLLimits.deposit: max per deposit exceeded")]
    MaxPerDepositExceeded {},

    #[error("StrategyBaseTVLLimits.deposit: max total deposits exceeded")]
    MaxTotalDepositsExceeded {},
}
