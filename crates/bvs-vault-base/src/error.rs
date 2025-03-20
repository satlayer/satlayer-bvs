use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {msg}")]
    Unauthorized { msg: String },

    #[error("Vault is not whitelisted")]
    NotWhitelisted {},

    #[error("Vault is validating, withdrawal must be queued")]
    Validating {},

    #[error("Insufficient: {msg}")]
    Insufficient { msg: String },

    #[error("Zero: {msg}")]
    Zero { msg: String },
}

impl VaultError {
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        VaultError::Unauthorized { msg: msg.into() }
    }

    pub fn insufficient(msg: impl Into<String>) -> Self {
        VaultError::Insufficient { msg: msg.into() }
    }

    pub fn zero(msg: impl Into<String>) -> Self {
        VaultError::Zero { msg: msg.into() }
    }
}
