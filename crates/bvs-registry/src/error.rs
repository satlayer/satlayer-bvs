use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Pauser(#[from] bvs_pauser::api::PauserError),

    #[error("{0}")]
    Ownership(#[from] bvs_library::ownership::OwnershipError),

    #[error("Not registered: {kind}")]
    NotRegistered { kind: String },
}

impl ContractError {
    pub fn not_registered(kind: impl Into<String>) -> Self {
        ContractError::NotRegistered { kind: kind.into() }
    }
}
