use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Registry(#[from] bvs_pauser::api::RegistryError),

    #[error("{0}")]
    Ownership(#[from] bvs_library::ownership::OwnershipError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Zero: {msg}")]
    Zero { msg: String },

    #[error("Insufficient: {msg}")]
    Insufficient { msg: String },
}

impl ContractError {
    pub fn zero(msg: impl Into<String>) -> Self {
        ContractError::Zero { msg: msg.into() }
    }

    pub fn insufficient(msg: impl Into<String>) -> Self {
        ContractError::Insufficient { msg: msg.into() }
    }
}
