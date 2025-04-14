use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RewardsError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Payment error: {0}")]
    Payment(#[from] PaymentError),

    #[error("Funds sent do not match the funds received")]
    FundsMismatch {},

    #[error("Merkle proof verification failed: {msg}")]
    InvalidProof { msg: String },

    #[error("Insufficient balance")]
    InsufficientBalance {},

    #[error("Rewards already claimed")]
    AlreadyClaimed {},
}
