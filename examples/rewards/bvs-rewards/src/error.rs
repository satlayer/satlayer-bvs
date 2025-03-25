use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RewardsError {
    #[error("{0}")]
    Std(#[from] StdError),
}
