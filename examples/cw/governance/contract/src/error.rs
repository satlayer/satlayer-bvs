use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Cw3(#[from] cw3_fixed_multisig::ContractError),

    #[error("Unauthorized")]
    Unauthorized {},
}
