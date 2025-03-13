use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

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

    #[error("BVSDirectory.setOperator: invalid input length")]
    InvalidInputLength {},

    #[error("BVSDirectory.setOperator: invalid input")]
    InvalidInput {},

    #[error("BVSDirectory.registerOperatorToBVS: operator not found")]
    OperatorNotFound {},

    #[error("BVSDirectory.registerOperatorToBVS: pubkey mismatch")]
    PubkeyMismatch {},
}
