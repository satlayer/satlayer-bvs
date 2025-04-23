use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{HexBinary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

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
    #[returns(BalanceResponse)]
    Balance { service: String, token: String },
    #[returns(ClaimRewardsResponse)]
    ClaimedRewards {
        service: String,
        token: String,
        earner: String,
    },
}

#[cw_serde]
pub struct DistributionRootResponse(pub String);

#[cw_serde]
pub struct BalanceResponse(pub Uint128);

#[cw_serde]
pub struct ClaimRewardsResponse(pub Uint128);

#[cw_serde]
pub enum RewardsType {
    Cw20,
    Bank,
}

#[cw_serde]
pub struct RewardDistribution {
    /// token refers to the address of the token contract (CW20) or denom string (Bank)
    ///
    /// ### CW20 Variant Warning
    ///
    /// Rewards that are not strictly CW20 compliant may cause unexpected behavior in token balances.
    /// For example, any token with a fee-on-transfer mechanism is not supported.
    /// Therefore, non-standard CW20 tokens are not supported.
    pub token: String,
    /// amount refers to the rewards to be transferred to the contract and distributed
    pub amount: Uint128,
}

#[cw_serde]
pub struct ClaimRewardsProof {
    /// root refers to the Merkle root of the Merkle tree
    pub root: HexBinary,
    /// proof is the Merkle proof of the user leaf in the Merkle tree
    pub proof: Vec<HexBinary>,
    /// leaf_index is the index of the user leaf in the Merkle tree
    pub leaf_index: u32,
    /// total_leaves_count is the total number of leaves in the Merkle tree
    pub total_leaves_count: u32,
}
