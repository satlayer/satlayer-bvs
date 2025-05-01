use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Request has already been responded to")]
    Responded {},

    #[error("Response not found")]
    ResponseNotFound {},

    #[error("Invalid prove, the output is correct")]
    InvalidProve {},
}
