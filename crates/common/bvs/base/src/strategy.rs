use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum QueryMsg {
    GetStakerStrategyShares { staker: String, strategy: String },
}

#[cw_serde]
pub struct StakerStrategySharesResponse {
    pub shares: Uint128,
}
