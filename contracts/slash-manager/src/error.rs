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

    #[error("SlashManager.submit_slash_request: invalid slash gignature")]
    InvalidSlashSignature {},

    #[error("SlashManager.submit_slash_request: invalid share")]
    InvalidShare {},

    #[error("SlashManager.submit_slash_request: invalid start time")]
    InvalidStartTime {},

    #[error("SlashManager.submit_slash_request: invalid end time")]
    InvalidEndTime {},

    #[error("SlashManager.submit_slash_request: invalid slash status")]
    InvalidSlashStatus {},

    #[error("SlashManager.cancel_slash_request: slash details not found")]
    SlashDetailsNotFound {},
}
