use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub minter: String,
    pub owner: String,
    pub strategy_manager: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    DepositWithMintAndStrategy {
        token: String,
        strategy: String,
        recipient: String,
        amount: Uint128,
        public_key: String,
        expiry: u64,
        signature: String,
    },
    SetMinter {
        minter: String,
    },
    SetStrategyManager {
        strategy_manager: String,
    },
    TransferOwnership {
        new_owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum StrategyExecuteMsg {
    DepositViaMirroredTokenWithSignature {
        strategy: String,
        token: String,
        amount: Uint128,
        staker: String,
        public_key: String,
        expiry: u64,
        signature: String,
    },
}
