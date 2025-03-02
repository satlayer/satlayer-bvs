use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, Api, Binary, StdResult, Uint128};
use sha2::{Digest, Sha256};

#[cw_serde]
pub struct DelegateParams {
    pub staker: Addr,
    pub operator: Addr,
}

#[cw_serde]
pub struct ExecuteDelegateParams {
    pub staker: String,
    pub operator: String,
}

#[cw_serde]
pub struct Withdrawal {
    pub staker: Addr,
    pub delegated_to: Addr,
    pub withdrawer: Addr,
    pub nonce: Uint128,
    pub start_block: u64,
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

pub fn calculate_withdrawal_root(withdrawal: &Withdrawal) -> StdResult<Binary> {
    let mut hasher = Sha256::new();
    hasher.update(to_json_binary(withdrawal)?.as_slice());
    Ok(Binary::from(hasher.finalize().as_slice()))
}
