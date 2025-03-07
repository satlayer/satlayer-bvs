use crate::merkle::{RewardsMerkleClaim, RewardsSubmission};
use crate::query::{
    CheckClaimResponse, GetCurrentClaimableDistributionRootResponse,
    GetCurrentDistributionRootResponse, GetDistributionRootAtIndexResponse,
    GetDistributionRootsLengthResponse, GetRootIndexFromHashResponse,
    OperatorCommissionBipsResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::HexBinary;

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
    CreateRewardsSubmission {
        rewards_submissions: Vec<RewardsSubmission>,
    },
    ProcessClaim {
        claim: RewardsMerkleClaim,
        recipient: String,
    },
    SubmitRoot {
        root: HexBinary,
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
    SetGlobalOperatorCommission {
        new_commission_bips: u16,
    },
    TransferOwnership {
        /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
        new_owner: String,
    },
    SetRewardsUpdater {
        addr: String,
    },
    SetRouting {
        strategy_manager: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OperatorCommissionBipsResponse)]
    OperatorCommissionBips { operator: String, service: String },

    #[returns(GetDistributionRootsLengthResponse)]
    GetDistributionRootsLength {},

    #[returns(GetCurrentDistributionRootResponse)]
    GetCurrentDistributionRoot {},

    #[returns(GetDistributionRootAtIndexResponse)]
    GetDistributionRootAtIndex { index: String },

    #[returns(GetCurrentClaimableDistributionRootResponse)]
    GetCurrentClaimableDistributionRoot {},

    #[returns(GetRootIndexFromHashResponse)]
    GetRootIndexFromHash { root_hash: HexBinary },

    #[returns(CheckClaimResponse)]
    CheckClaim { claim: RewardsMerkleClaim },
}

#[cw_serde]
pub struct DistributionRoot {
    pub root: HexBinary,
    pub rewards_calculation_end_timestamp: u64,
    pub activated_at: u64,
    pub disabled: bool,
}
