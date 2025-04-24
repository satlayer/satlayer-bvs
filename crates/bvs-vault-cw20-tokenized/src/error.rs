use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Vault(#[from] bvs_vault_base::error::VaultError),

    #[error("{0}")]
    Pauser(#[from] bvs_pauser::api::PauserError),

    #[error("{0}")]
    Cw20Base(#[from] cw20_base::ContractError),
}
