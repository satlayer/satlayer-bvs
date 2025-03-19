use bvs_vault_base;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Vault(#[from] bvs_vault_base::error::VaultError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0}")]
    Ownership(#[from] bvs_library::ownership::OwnershipError),
}
