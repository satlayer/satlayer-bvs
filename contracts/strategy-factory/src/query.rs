use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct StrategyResponse {
    pub strategy: Addr,
}

#[cw_serde]
pub struct BlacklistStatusResponse {
    pub is_blacklisted: bool,
}
