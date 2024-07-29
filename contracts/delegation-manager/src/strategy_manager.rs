// strategy_manager.rs

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_storage_plus::{Item, Map};

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
        third_party_transfers_forbidden_values: Vec<bool>,
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
    RemoveShares {
        staker: Addr,
        strategy: Addr,
        shares: Uint128,
    },
    WithdrawSharesAsTokens {
        recipient: Addr,
        strategy: Addr,
        shares: Uint128,
        token: Addr,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetDeposits { staker: Addr },
    StakerStrategyListLength { staker: Addr },
}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: String,
    pub salt: String,
    pub expiry: Uint64,
}

#[cw_serde]
pub struct StrategyManagerState {
    pub delegation_manager: Addr,
    pub slasher: Addr,
}

pub const STRATEGY_MANAGER_STATE: Item<StrategyManagerState> = Item::new("strategy_manager_state");
pub const STRATEGY_WHITELISTER: Item<Addr> = Item::new("strategy_whitelister");
pub const STRATEGY_WHITELIST: Map<&Addr, bool> = Map::new("strategy_whitelist");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const STAKER_STRATEGY_SHARES: Map<(&Addr, &Addr), Uint128> = Map::new("staker_strategy_shares");
pub const STAKER_STRATEGY_LIST: Map<&Addr, Vec<Addr>> = Map::new("staker_strategy_list");
pub const MAX_STAKER_STRATEGY_LIST_LENGTH: usize = 10;
pub const THIRD_PARTY_TRANSFERS_FORBIDDEN: Map<&Addr, bool> = Map::new("third_party_transfers_forbidden");
pub const NONCES: Map<&Addr, u64> = Map::new("nonces");
