use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub strategy_code_id: u64,
    pub strategy_manager: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u8,
}

#[cw_serde]
pub enum ExecuteMsg {
    DeployNewStrategy {
        token: String,
        pauser: String,
        unpauser: String,
    },
    UpdateConfig {
        new_owner: String,
        strategy_code_id: u64,
    },
    BlacklistTokens {
        tokens: Vec<String>,
    },
    RemoveStrategiesFromWhitelist {
        strategies: Vec<String>,
    },
    SetThirdPartyTransfersForBidden {
        strategy: String,
        value: bool,
    },
    WhitelistStrategies {
        strategies_to_whitelist: Vec<String>,
        third_party_transfers_forbidden_values: Vec<bool>,
    },
    SetStrategyManager {
        new_strategy_manager: String,
    },
    TransferOwnership {
        new_owner: String,
    },
    Pause {},
    Unpause {},
    SetPauser {
        new_pauser: String,
    },
    SetUnpauser {
        new_unpauser: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetStrategy { token: String },
    IsTokenBlacklisted { token: String },
}

#[cw_serde]
pub struct MigrateMsg {}
