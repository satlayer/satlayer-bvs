use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Statebank: Bvs contract is not registered")]
    BvsContractNotRegistered {},

    #[error("Statebank.migrate: unauthorized")]
    Unauthorized {},
}
