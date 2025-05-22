use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Cw3Flex(#[from] cw3_flex_multisig::ContractError),

    #[error("Unauthorized")]
    Unauthorized {},
}
