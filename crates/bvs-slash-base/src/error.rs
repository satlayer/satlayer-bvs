use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SlasherError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {msg}")]
    Unauthorized { msg: String },

    #[error("Invalid Offender: {msg}")]
    InvalidOffender { msg: String },
}

impl SlasherError {
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        SlasherError::Unauthorized { msg: msg.into() }
    }
}
