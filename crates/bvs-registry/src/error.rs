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

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Service has been registered")]
    ServiceRegistered {},

    #[error("Operator has been registered")]
    OperatorRegistered {},

    #[error("Invalid registration status: {msg}")]
    InvalidRegistrationStatus { msg: String },

    #[error("Invalid slashing parameters: {msg}")]
    InvalidSlashingParameters { msg: String },

    #[error("Invalid slashing opt-in: {msg}")]
    InvalidSlashingOptIn { msg: String },

    #[error("Invalid slashing request: {msg}")]
    InvalidSlashingRequest { msg: String },
}
