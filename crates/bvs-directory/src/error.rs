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

    #[error("BVSDirectory.registerOperatorToBVS: operator signature expired")]
    SignatureExpired {},

    #[error("BVSDirectory.registerOperatorToBVS: operator already registered")]
    OperatorAlreadyRegistered {},

    #[error("BVSDirectory.registerOperatorToBVS: salt already spent")]
    SaltAlreadySpent {},

    #[error("BVSDirectory.deregisterOperatorFromBVS: operator not registered yet")]
    OperatorNotRegistered {},

    #[error("BVSDirectory.registerOperatorToBVS: invalid signature")]
    InvalidSignature {},

    #[error("BVSDirectory.transferOwnership: unauthorized")]
    Unauthorized {},

    #[error("DelegationManager.IsOperator: operator not registered yet from delegation manager")]
    OperatorNotRegisteredFromDelegationManager {},
}
