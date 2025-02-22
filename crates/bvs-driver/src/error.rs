use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("BvsDriver: Bvs contract is not registered")]
    BvsContractNotRegistered {},

    #[error("BvsDriver: unauthorized")]
    Unauthorized {},

    #[error("BvsDriver: no pending owner")]
    NoPendingOwner {},

    #[error("BvsDriver.onlyDirectory: not BVS directory")]
    NotBVSDirectory {},
}
