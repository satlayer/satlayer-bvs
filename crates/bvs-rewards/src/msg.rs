use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{HexBinary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner of this contract
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    DistributeRewards {
        merkle_root: HexBinary,
        reward_distribution: RewardDistribution,
        reward_type: RewardsType,
    },
    ClaimRewards {
        claim_rewards_proof: ClaimRewardsProof,
        reward_type: RewardsType,
        service: String,
        /// token refers to the address of the token contract (CW20) or denom string (Bank)
        token: String,
        /// amount refers to the total amount of rewards accrued to the user
        amount: Uint128,
        recipient: String,
    },
    TransferOwnership {
        /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
        new_owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DistributionRootResponse)]
    DistributionRoot { service: String, token: String },
}

#[cw_serde]
pub struct DistributionRootResponse(pub String);

#[cw_serde]
pub enum RewardsType {
    CW20,
    Bank,
}

#[cw_serde]
pub struct RewardDistribution {
    /// token refers to the address of the token contract (CW20) or denom string (Bank)
    pub token: String,
    /// amount refers to the additional rewards to be transferred to the contract and distributed
    pub amount: Uint128,
}

#[cw_serde]
pub struct ClaimRewardsProof {
    /// root refers to the Merkle root of the Merkle tree
    pub root: HexBinary,
    /// proof is the Merkle proof of the user leaf in the Merkle tree
    pub proof: Vec<HexBinary>,
    /// leaf_index is the index of the user leaf in the Merkle tree
    pub leaf_index: Uint128,
    /// total_leaves_count is the total number of leaves in the Merkle tree
    pub total_leaves_count: Uint128,
}

#[cfg(test)]
mod tests {}
