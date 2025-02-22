use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("StateBank: contract is not registered")]
    BvsContractNotRegistered {},

    #[error("StateBank: unauthorized")]
    Unauthorized {},

    #[error("StateBank.acceptOwnership: no pending owner")]
    NoPendingOwner {},

    #[error("StateBank: not BVS directory")]
    NotBVSDirectory {},
}
