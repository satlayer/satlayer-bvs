use crate::query::{
    CalculateEarnerLeafHashResponse, CalculateTokenLeafHashResponse, CheckClaimResponse,
    GetCurrentClaimableDistributionRootResponse, GetCurrentDistributionRootResponse,
    GetDistributionRootAtIndexResponse, GetDistributionRootsLengthResponse,
    GetRootIndexFromHashResponse, MerkleizeLeavesResponse, OperatorCommissionBipsResponse,
};
use crate::utils::{ExecuteRewardsMerkleClaim, RewardsSubmission};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
    pub calculation_interval_seconds: u64,
    pub max_rewards_duration: u64,
    pub max_retroactive_length: u64,
    pub max_future_length: u64,
    pub genesis_rewards_timestamp: u64,
    pub activation_delay: u32,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    CreateBvsRewardsSubmission {
        rewards_submissions: Vec<RewardsSubmission>,
    },
    CreateRewardsForAllSubmission {
        rewards_submissions: Vec<RewardsSubmission>,
    },
    ProcessClaim {
        claim: ExecuteRewardsMerkleClaim,
        recipient: String,
    },
    SubmitRoot {
        root: String,
        rewards_calculation_end_timestamp: u64,
    },
    DisableRoot {
        root_index: u64,
    },
    SetClaimerFor {
        claimer: String,
    },
    SetActivationDelay {
        new_activation_delay: u32,
    },
    SetRewardsForAllSubmitter {
        submitter: String,
        new_value: bool,
    },
    SetGlobalOperatorCommission {
        new_commission_bips: u16,
    },
    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
    },
    SetRewardsUpdater {
        addr: String,
    },
    SetRouting {
        delegation_manager: String,
        strategy_manager: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CalculateEarnerLeafHashResponse)]
    CalculateEarnerLeafHash {
        earner: String,
        earner_token_root: String,
    },

    #[returns(CalculateTokenLeafHashResponse)]
    CalculateTokenLeafHash {
        token: String,
        cumulative_earnings: Uint128,
    },

    #[returns(OperatorCommissionBipsResponse)]
    OperatorCommissionBips { operator: String, bvs: String },

    #[returns(GetDistributionRootsLengthResponse)]
    GetDistributionRootsLength {},

    #[returns(GetCurrentDistributionRootResponse)]
    GetCurrentDistributionRoot {},

    #[returns(GetDistributionRootAtIndexResponse)]
    GetDistributionRootAtIndex { index: String },

    #[returns(GetCurrentClaimableDistributionRootResponse)]
    GetCurrentClaimableDistributionRoot {},

    #[returns(GetRootIndexFromHashResponse)]
    GetRootIndexFromHash { root_hash: String },

    #[returns(MerkleizeLeavesResponse)]
    MerkleizeLeaves { leaves: Vec<String> },

    #[returns(CheckClaimResponse)]
    CheckClaim { claim: ExecuteRewardsMerkleClaim },
}

#[cw_serde]
pub struct DistributionRoot {
    pub root: Binary,
    pub rewards_calculation_end_timestamp: u64,
    pub activated_at: u64,
    pub disabled: bool,
}
