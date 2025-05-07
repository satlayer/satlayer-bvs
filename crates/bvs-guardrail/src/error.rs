use bvs_library::ownership::OwnershipError;
use cosmwasm_std::{OverflowError, StdError};
use cw_utils::ThresholdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("{0}")]
    Threshold(#[from] ThresholdError),

    #[error("{0}")]
    Cw4Group(#[from] cw4_group::ContractError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Required weight cannot be zero")]
    ZeroWeight {},

    #[error("Not possible to reach required (passing) weight")]
    UnreachableWeight {},

    #[error("No voters")]
    NoVoters {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Proposal is not open")]
    NotOpen {},

    #[error("Proposal voting period has expired")]
    Expired {},

    #[error("Proposal must expire before you can close it")]
    NotExpired {},

    #[error("Wrong expiration option")]
    WrongExpiration {},

    #[error("Already voted on this proposal")]
    AlreadyVoted {},

    #[error("Proposal must have passed and not yet been executed")]
    WrongExecuteStatus {},

    #[error("Cannot close completed or passed proposals")]
    WrongCloseStatus {},

    #[error("Proposal already exists")]
    ProposalAlreadyExists,

    #[error("Proposal does not exist")]
    ProposalNotFound {},
}
