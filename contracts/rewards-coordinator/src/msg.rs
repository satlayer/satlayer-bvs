use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint64, Binary, Uint128};
use crate::utils::{RewardsSubmission, ExeuteRewardsMerkleClaim};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: Addr,
    pub rewards_updater: Addr,
    pub calculation_interval_seconds: u64,
    pub max_rewards_duration: u64,
    pub max_retroactive_length: u64,
    pub max_future_length: u64,
    pub genesis_rewards_timestamp: u64,
    pub delegation_manager: Addr,
    pub strategy_manager: Addr,
    pub activation_delay: u32, 
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateAvsRewardsSubmission {
        rewards_submissions: Vec<RewardsSubmission>,
    },
    CreateRewardsForAllSubmission {
        rewards_submissions: Vec<RewardsSubmission>,
    },
    ProcessClaim {
        claim: ExeuteRewardsMerkleClaim,
        recipient: Addr,
    },
    SubmitRoot {
        root: String,
        rewards_calculation_end_timestamp: Uint64,
    },
    DisableRoot {
        root_index: u64,
    },
    SetClaimerFor {
        claimer: Addr,
    },
    SetActivationDelay {
        new_activation_delay: u32,
    },
    SetRewardsUpdater {
        new_updater: Addr,
    },
    SetRewardsForAllSubmitter {
        submitter: Addr,
        new_value: bool,
    },
    SetGlobalOperatorCommission {
        new_commission_bips: u16,
    },
    TransferOwnership {
        new_owner: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Binary)]
    CalculateEarnerLeafHash { earner: String, earner_token_root: String },

    #[returns(Binary)]
    CalculateTokenLeafHash { token: String, cumulative_earnings: String },

    #[returns(Uint128)]
    QueryOperatorCommissionBips { operator: String, avs: String },

    #[returns(Uint64)]
    GetDistributionRootsLength {},

    #[returns(DistributionRoot)]
    GetCurrentDistributionRoot {},  

    #[returns(DistributionRoot)]
    GetDistributionRootAtIndex { index: String },

    #[returns(DistributionRoot)]
    GetCurrentClaimableDistributionRoot {},

    #[returns(u32)]
    GetRootIndexFromHash { root_hash: String },

    #[returns(Binary)]
    CalculateDomainSeparator { chain_id: String, contract_addr: String },

    #[returns(Binary)]
    MerkleizeLeaves { leaves: Vec<String> },

    #[returns(bool)]
    CheckClaim { claim: ExeuteRewardsMerkleClaim },
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