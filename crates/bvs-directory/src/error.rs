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

    #[error("Service already registered")]
    ServiceRegistered {},

    #[error("Service not found")]
    ServiceNotFound {},

    #[error("Invalid registration status: {msg}")]
    InvalidRegistrationStatus { msg: String },

    #[error("Operator not found in delegation {msg}")]
    OperatorNotFound { msg: String },
}
