use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64, Binary};

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
        public_key: Binary,
        expiry: Uint64,
        signature: Binary,
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
    pub signature: Binary,
    pub salt: Binary,
    pub expiry: Uint64,
}