use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Pauser(#[from] bvs_pauser::api::PauserError),

    #[error("{0}")]
    Ownership(#[from] bvs_library::ownership::OwnershipError),

    #[error("Vault error: {msg}")]
    VaultError { msg: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Code id not found")]
    CodeIdNotFound {},
}
