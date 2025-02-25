use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum ExecuteMsg {
    RemoveStrategiesFromWhitelist {
        strategies: Vec<String>,
    },
    SetThirdPartyTransfersForbidden {
        strategy: String,
        value: bool,
    },
    AddStrategiesToWhitelist {
        strategies: Vec<String>,
        third_party_transfers_forbidden_values: Vec<bool>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetStakerStrategyShares { staker: String, strategy: String },
}

#[cw_serde]
pub struct StakerStrategySharesResponse {
    pub shares: Uint128,
}
