use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("MirroredToken.onlyOwner: Unauthorized")]
    NotOwnerUnauthorized {},

    #[error("MirroredToken.onlyMinter: Unauthorized")]
    NotMinterUnauthorized {},
}
