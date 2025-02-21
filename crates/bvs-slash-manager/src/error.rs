use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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

    #[error("SlashManager.submit_slash_request: operator not registered")]
    OperatorNotRegistered {},

    #[error("SlashManager.execute_slash_request: invalid signature")]
    InvalidSignature {},

    #[error("SlashManager.ExecuteSlashRequest: invalid validator")]
    InvalidValidator {},

    #[error("SlashManager.execute_slash_request: no stakers under operator")]
    NoStakersUnderOperator {},

    #[error("SlashManager.execute_slash_request: slash share too small")]
    SlashShareTooSmall {},

    #[error("SlashManager.execute_slash_request: overflow")]
    Overflow {},

    #[error("SlashManager.execute_slash_request: underflow")]
    Underflow {},

    #[error("SlashManager.execute_slash_request: insufficient shares for staker")]
    InsufficientSharesForStaker { staker: String },

    #[error("SlashManager.execute_slash_request: signature empty")]
    SignatureEmpty {},

    #[error("SlashManager.execute_slash_request: signature not enough")]
    SignatureNotEnough {},

    #[error("SlashManager.execute_slash_request: empty validator list")]
    EmptyValidatorList {},

    #[error("SlashManager.execute_slash_request: duplicate validator public key")]
    DuplicateValidatorPublicKey {},
}
