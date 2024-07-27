use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub delegation_manager: Addr,
    pub slasher: Addr,
    pub initial_strategy_whitelister: Addr,
    pub initial_owner: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddStrategiesToWhitelist {
        strategies: Vec<Addr>,
    },
    RemoveStrategiesFromWhitelist {
        strategies: Vec<Addr>,
    },
    SetStrategyWhitelister {
        new_strategy_whitelister: Addr,
    },
    DepositIntoStrategy {
        strategy: Addr,
        token: Addr,
        amount: Uint128,
    },
    SetThirdPartyTransfersForbidden {
        strategy: Addr,
        value: bool,
    },
    DepositIntoStrategyWithSignature {
        strategy: Addr,
        token: Addr,
        amount: Uint128,
        staker: Addr,
        expiry: Uint64,
        signature: String,
    },
}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: String,
    pub salt: String,
    pub expiry: Uint64,
}
