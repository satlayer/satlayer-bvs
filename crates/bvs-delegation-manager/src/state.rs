use crate::msg::OperatorDetails;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const OPERATOR_DETAILS: Map<&Addr, OperatorDetails> = Map::new("operator_details");
pub const DELEGATED_TO: Map<&Addr, Addr> = Map::new("delegated_to");
pub const OPERATOR_SHARES: Map<(&Addr, &Addr), Uint128> = Map::new("operator_shares");
pub const PENDING_WITHDRAWALS: Map<&[u8], bool> = Map::new("pending_withdrawals");
pub const STRATEGY_WITHDRAWAL_DELAY_BLOCKS: Map<&Addr, u64> =
    Map::new("strategy_withdrawal_delay_blocks");
pub const MIN_WITHDRAWAL_DELAY_BLOCKS: Item<u64> = Item::new("min_withdrawal_delay_blocks");
pub const CUMULATIVE_WITHDRAWALS_QUEUED: Map<&Addr, Uint128> =
    Map::new("cumulative_withdrawals_queued");
