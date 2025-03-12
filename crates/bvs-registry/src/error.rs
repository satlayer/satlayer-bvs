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

    #[error("Service not found")]
    ServiceNotFound {},

    #[error("Invalid registration status: {msg}")]
    InvalidRegistrationStatus { msg: String },

    #[error("Operator is not registered")]
    OperatorNotFound {},

    #[error("Operator update error: {msg}")]
    OperatorUpdate { msg: String },
}
