use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Registry(#[from] bvs_registry::api::RegistryError),

    #[error("{0}")]
    Ownership(#[from] bvs_library::ownership::OwnershipError),

    #[error("StrategyBase: unauthorized")]
    Unauthorized {},

    #[error("StrategyBase.deposit: new_shares cannot be zero")]
    ZeroNewShares {},

    #[error("StrategyBase.withdraw: amount to send cannot be zero")]
    ZeroAmountToSend {},

    #[error("StrategyBase.withdraw: insufficient shares")]
    InsufficientShares {},

    #[error("StrategyBase: invalid token")]
    InvalidToken {},

    #[error("StrategyBase.withdraw: insufficient balance")]
    InsufficientBalance {},
}
