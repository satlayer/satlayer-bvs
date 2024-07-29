use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct DelegationManagerState {
    pub strategy_manager: Addr,
    pub slasher: Addr,
}

#[cw_serde]
pub struct OperatorDetails {
    pub __deprecated_earnings_receiver: Addr,
    pub delegation_approver: Addr,
    pub staker_opt_out_window_blocks: u64,
}

pub const DELEGATION_MANAGER_STATE: Item<DelegationManagerState> = Item::new("delegation_manager_state");
pub const OPERATOR_DETAILS: Map<&Addr, OperatorDetails> = Map::new("operator_details");
pub const DELEGATED_TO: Map<&Addr, Addr> = Map::new("delegated_to");
pub const OPERATOR_SHARES: Map<(&Addr, &Addr), Uint128> = Map::new("operator_shares");
pub const STAKER_NONCE: Map<&Addr, u64> = Map::new("staker_nonce");
// pub const PENDING_WITHDRAWALS: Map<&Addr, Withdrawal> = Map::new("pending_withdrawals");
pub const STRATEGY_WITHDRAWAL_DELAY_BLOCKS: Map<&Addr, u64> = Map::new("strategy_withdrawal_delay_blocks");
pub const STRATEGY_MANAGER: Item<Addr> = Item::new("strategy_manager");
pub const SLASHER: Item<Addr> = Item::new("slasher");
pub const EIGEN_POD_MANAGER: Item<Addr> = Item::new("eigen_pod_manager");
pub const MIN_WITHDRAWAL_DELAY_BLOCKS: Item<u64> = Item::new("min_withdrawal_delay_blocks");
pub const CUMULATIVE_WITHDRAWALS_QUEUED: Map<&Addr, u128> = Map::new("cumulative_withdrawals_queued");
pub const DOMAIN_SEPARATOR: Item<Binary> = Item::new("domain_separator");
pub const OWNER: Item<Addr> = Item::new("owner");
pub const DELEGATION_APPROVER_SALT_SPENT: Map<(&Addr, &String), bool> = Map::new("delegation_approver_salt_spent");