// This file was automatically generated from rewards/schema.json.
// DO NOT MODIFY IT BY HAND.

/**
 * This is a wrapper around Vec<u8> to add hex de/serialization with serde. It also adds
 * some helper methods to help encode inline.
 *
 * This is similar to `cosmwasm_std::Binary` but uses hex. See also
 * <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 *
 * root refers to the Merkle root of the Merkle tree
 *
 * amount refers to the additional rewards to be transferred to the contract and
 * distributed
 *
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that
 * the full u128 range can be used for clients that convert JSON numbers to floats, like
 * JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(),
 * 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 *
 * amount refers to the total amount of rewards accrued to the user
 */
type BalanceResponse = string;

/**
 * This is a wrapper around Vec<u8> to add hex de/serialization with serde. It also adds
 * some helper methods to help encode inline.
 *
 * This is similar to `cosmwasm_std::Binary` but uses hex. See also
 * <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 *
 * root refers to the Merkle root of the Merkle tree
 *
 * amount refers to the additional rewards to be transferred to the contract and
 * distributed
 *
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that
 * the full u128 range can be used for clients that convert JSON numbers to floats, like
 * JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(),
 * 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 *
 * amount refers to the total amount of rewards accrued to the user
 */
type ClaimRewardsResponse = string;

/**
 * This is a wrapper around Vec<u8> to add hex de/serialization with serde. It also adds
 * some helper methods to help encode inline.
 *
 * This is similar to `cosmwasm_std::Binary` but uses hex. See also
 * <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 *
 * root refers to the Merkle root of the Merkle tree
 *
 * amount refers to the additional rewards to be transferred to the contract and
 * distributed
 *
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that
 * the full u128 range can be used for clients that convert JSON numbers to floats, like
 * JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(),
 * 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 *
 * amount refers to the total amount of rewards accrued to the user
 */
type DistributionRootResponse = string;

export interface InstantiateMsg {
  /**
   * Owner of this contract
   */
  owner: string;
}

export interface ExecuteMsg {
  distribute_rewards?: DistributeRewards;
  claim_rewards?: ClaimRewards;
  transfer_ownership?: TransferOwnership;
}

export interface ClaimRewards {
  /**
   * amount refers to the total amount of rewards accrued to the user
   */
  amount: string;
  claim_rewards_proof: ClaimRewardsProof;
  recipient: string;
  reward_type: RewardsType;
  service: string;
  /**
   * token refers to the address of the token contract (CW20) or denom string (Bank)
   */
  token: string;
}

export interface ClaimRewardsProof {
  /**
   * leaf_index is the index of the user leaf in the Merkle tree
   */
  leaf_index: number;
  /**
   * proof is the Merkle proof of the user leaf in the Merkle tree
   */
  proof: string[];
  /**
   * root refers to the Merkle root of the Merkle tree
   */
  root: string;
  /**
   * total_leaves_count is the total number of leaves in the Merkle tree
   */
  total_leaves_count: number;
}

export enum RewardsType {
  Bank = "bank",
  Cw20 = "cw20",
}

export interface DistributeRewards {
  merkle_root: string;
  reward_distribution: RewardDistribution;
  reward_type: RewardsType;
}

export interface RewardDistribution {
  /**
   * amount refers to the additional rewards to be transferred to the contract and distributed
   */
  amount: string;
  /**
   * token refers to the address of the token contract (CW20) or denom string (Bank)
   */
  token: string;
}

export interface TransferOwnership {
  /**
   * See [`bvs_library::ownership::transfer_ownership`] for more information on this field
   */
  new_owner: string;
}

export interface QueryMsg {
  distribution_root?: DistributionRoot;
  balance?: Balance;
  claimed_rewards?: ClaimedRewards;
}

export interface Balance {
  service: string;
  token: string;
}

export interface ClaimedRewards {
  earner: string;
  service: string;
  token: string;
}

export interface DistributionRoot {
  service: string;
  token: string;
}
