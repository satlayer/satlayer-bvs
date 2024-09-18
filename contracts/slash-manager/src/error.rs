use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("SlashManager: unauthorized")]
    Unauthorized {},

    #[error("SlashManager.set_slash_validator: invalid input length")]
    InvalidInputLength {},
}
