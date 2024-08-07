use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint64, Binary};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: Addr,
    pub rewards_updater: Addr,
    pub calculation_interval_seconds: Uint64,
    pub max_rewards_duration: Uint64,
    pub max_retroactive_length: Uint64,
    pub max_future_length: Uint64,
    pub genesis_rewards_timestamp: Uint64,
    pub delegation_manager: Addr,
    pub strategy_manager: Addr,
    pub activation_delay: u32, 
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateRewardsSubmission {
        submissions: Vec<MerkleRootSubmission>,
    },
    ProcessClaim {
        claim: MerkleRootSubmission,
        recipient: Addr,
    },
    SubmitRoot {
        root: String,
        rewards_calculation_end_timestamp: Uint64,
    },
    DisableRoot {
        root_index: u64,
    },
    TransferOwnership {
        new_owner: Addr,
    },
    SetRewardsUpdater {
        new_updater: Addr,
    },
}

#[cw_serde]
pub enum QueryMsg {
    CalculateEarnerLeafHash { earner: String, earner_token_root: String },
    CalculateTokenLeafHash { token: String, cumulative_earnings: String },
    QueryOperatorCommissionBips { operator: String, avs: String},
    GetDistributionRootsLength {},
    GetCurrentDistributionRoot {},  
    GetDistributionRootAtIndex { index: String },
    GetCurrentClaimableDistributionRoot {},
    GetRootIndexFromHash { root_hash: String },
    CalculateDomainSeparator { chain_id: String, contract_addr: String },
}

#[cw_serde]
pub struct MerkleRootSubmission {
    pub earner: Addr,
    pub token: String,
    pub amount: Uint64,
    pub start_timestamp: Uint64,
}

#[cw_serde]
pub struct RewardsStatusResponse {
    pub earner: Addr,
    pub token: String,
    pub claimed: u128,
}

#[cw_serde]
pub struct DistributionRoot {
    pub root: Binary,
    pub rewards_calculation_end_timestamp: Uint64,
    pub activated_at: Uint64,
    pub disabled: bool,
}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: Binary,
    pub salt: Binary,
    pub expiry: Uint64,
}