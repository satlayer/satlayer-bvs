use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("BVSSquaring: unauthorized")]
    Unauthorized {},

    #[error("BVSSquaring: task result already submitted")]
    ResultSubmitted {},

    #[error("BVSSquaring: no value found")]
    NoValueFound {},
}
