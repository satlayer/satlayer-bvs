use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("AVSDirectory.registerOperatorToAVS: operator signature expired")]
    SignatureExpired {},

    #[error("AVSDirectory.registerOperatorToAVS: operator already registered")]
    OperatorAlreadyRegistered {},

    #[error("AVSDirectory.registerOperatorToAVS: salt already spent")]
    SaltAlreadySpent {},

    #[error("AVSDirectory.registerOperatorToAVS: operator not registered to EigenLayer yet")]
    OperatorNotRegistered {},

    #[error("AVSDirectory.registerOperatorToAVS: invalid signature")]
    InvalidSignature {},
}