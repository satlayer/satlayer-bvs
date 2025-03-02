use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Registry(#[from] bvs_registry::api::RegistryError),

    #[error("{0}")]
    Ownership(#[from] bvs_library::ownership::OwnershipError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("operator signature expired")]
    SignatureExpired {},

    #[error("operator already registered")]
    OperatorAlreadyRegistered {},

    #[error("salt already spent")]
    SaltAlreadySpent {},

    #[error("operator not registered yet")]
    OperatorNotRegistered {},

    #[error("invalid signature")]
    InvalidSignature {},

    #[error("operator not registered yet from delegation manager")]
    OperatorNotRegisteredFromDelegationManager {},
}
