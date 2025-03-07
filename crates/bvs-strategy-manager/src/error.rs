use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Registry(#[from] bvs_pauser::api::RegistryError),

    #[error("{0}")]
    Ownership(#[from] bvs_library::ownership::OwnershipError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Strategy is not whitelisted")]
    NotWhitelisted {},

    #[error("Invalid strategy: {msg}")]
    InvalidStrategy { msg: String },

    #[error("StrategyManager: invalid shares")]
    InvalidShares {},

    #[error("StrategyManager.add_shares: max strategy list length exceeded")]
    MaxStrategyListLengthExceeded {},

    #[error("StrategyManager.remove_strategy_from_staker_strategy_list: strategy not found")]
    StrategyNotFound {},
}
