use crate::msg::DistributionRoot;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

/// Stores the history of [`submit_root`](crate::contract::submit_root) with <Index, DistributionRoot>,
/// where Index is the [`DISTRIBUTION_ROOTS_COUNT`] at the point of submission
pub const DISTRIBUTION_ROOTS: Map<u64, DistributionRoot> = Map::new("distribution_roots");

/// Stores the running count of [`DistributionRoot`] submitted
pub const DISTRIBUTION_ROOTS_COUNT: Item<u64> = Item::new("distribution_roots_count");

/// Stores the total amount claimed for each `(earner, token)` pair
pub const CUMULATIVE_CLAIMED: Map<(&Addr, &Addr), Uint128> = Map::new("cumulative_claimed");

/// Stores the running count of [`RewardsSubmission`](crate::utils::RewardsSubmission) submitted per `Service`
pub const SUBMISSION_NONCE: Map<&Addr, u64> = Map::new("submission_nonce");

/// Stores the address permissioned to call [`create_rewards_for_all_submission`](crate::contract::create_rewards_for_all_submission)
pub const REWARDS_FOR_ALL_SUBMITTER: Map<&Addr, bool> = Map::new("rewards_for_all_submitter");

/// Stores the delay from the time of reward submission to the time of activation / claimable time
pub const ACTIVATION_DELAY: Item<u32> = Item::new("activation_delay");

/// Stores `(Service, RewardsSubmissionHash)` pairs that have been submitted
pub const IS_BVS_REWARDS_SUBMISSION_HASH: Map<(&Addr, &[u8]), bool> =
    Map::new("bvs_rewards_submission_hash");

/// Stores the base unit of [`RewardsSubmission::duration`](field@crate::utils::RewardsSubmission::duration) (in seconds)
pub const CALCULATION_INTERVAL_SECONDS: Item<u64> = Item::new("calculation_interval_seconds");

/// Stores the maximum duration of [`RewardsSubmission`](crate::utils::RewardsSubmission) activation (in base unit)
pub const MAX_REWARDS_DURATION: Item<u64> = Item::new("max_rewards_duration");

/// Stores the maximum retroactive length of [`RewardsSubmission`](crate::utils::RewardsSubmission) activation (in seconds)
pub const MAX_RETROACTIVE_LENGTH: Item<u64> = Item::new("max_retroactive_length");

/// Stores the maximum future length of [`RewardsSubmission`](crate::utils::RewardsSubmission) activation (in seconds)
pub const MAX_FUTURE_LENGTH: Item<u64> = Item::new("max_future_length");

// Stores the timestamp of the genesis rewards (in seconds)
pub const GENESIS_REWARDS_TIMESTAMP: Item<u64> = Item::new("genesis_rewards_timestamp");

/// Stores the latest timestamp used in the most recent rewards calculation (in seconds)
pub const CURR_REWARDS_CALCULATION_END_TIMESTAMP: Item<u64> =
    Item::new("curr_rewards_calculation_end_timestamp");

/// stores the default operator commission (in basis points)
pub const GLOBAL_OPERATOR_COMMISSION_BIPS: Item<u16> = Item::new("global_operator_commission_bips");

/// Stores the permissioned address to claim for an `earner`
pub const CLAIMER_FOR: Map<&Addr, Addr> = Map::new("claimer_for");
