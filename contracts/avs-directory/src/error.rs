use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Signature expired")]
    SignatureExpired {},

    #[error("Operator already registered")]
    OperatorAlreadyRegistered {},

    #[error("Salt already spent")]
    SaltAlreadySpent {},

    #[error("Operator not registered")]
    OperatorNotRegistered {},

    #[error("Invalid signature")]
    InvalidSignature {},
}
