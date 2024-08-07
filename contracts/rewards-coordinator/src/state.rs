use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};
use crate::msg::DistributionRoot;

pub const OWNER: Item<Addr> = Item::new("owner");
pub const REWARDS_UPDATER: Item<Addr> = Item::new("rewards_updater");
pub const DISTRIBUTION_ROOTS: Map<u64, DistributionRoot> = Map::new("distribution_roots");
pub const DISTRIBUTION_ROOTS_COUNT: Item<u64> = Item::new("distribution_roots_count");
pub const CUMULATIVE_CLAIMED: Map<(Addr, String), u128> = Map::new("cumulative_claimed");
pub const SUBMISSION_NONCE: Map<Addr, u64> = Map::new("submission_nonce");
pub const REWARDS_FOR_ALL_SUBMITTER: Map<Addr, bool> = Map::new("rewards_for_all_submitter");
pub const ACTIVATION_DELAY: Item<u32> = Item::new("activation_delay");
pub const IS_AVS_REWARDS_SUBMISSION_HASH: Map<(Addr, Vec<u8>), bool> = Map::new("avs_rewards_submission_hash");

pub const CALCULATION_INTERVAL_SECONDS: Item<u64> = Item::new("calculation_interval_seconds");
pub const MAX_REWARDS_DURATION: Item<u64> = Item::new("max_rewards_duration");
pub const MAX_RETROACTIVE_LENGTH: Item<u64> = Item::new("max_retroactive_length");
pub const MAX_FUTURE_LENGTH: Item<u64> = Item::new("max_future_length");
pub const GENESIS_REWARDS_TIMESTAMP: Item<u64> = Item::new("genesis_rewards_timestamp");
pub const CURR_REWARDS_CALCULATION_END_TIMESTAMP: Item<Uint64> = Item::new("curr_rewards_calculation_end_timestamp");
pub const DELEGATION_MANAGER: Item<Addr> = Item::new("delegation_manager");
pub const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");
pub const GLOBAL_OPERATOR_COMMISSION_BIPS: Item<u16> = Item::new("global_operator_commission_bips");
pub const CLAIMER_FOR: Map<Addr, Addr> = Map::new("claimer_for");



