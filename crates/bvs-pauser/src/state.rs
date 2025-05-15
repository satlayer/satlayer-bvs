use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// The PAUSED_PAUSED_CONTRACT_METHOD state contains the information for which method are paused on a particular contract
/// Key (composite): (contract_addr, method_msg)
/// Value: u64, the block height at which the method was paused
pub(crate) const PAUSED_CONTRACT_METHOD: Map<(&Addr, &String), u64> =
    Map::new("paused_contract_method");

/// The PAUSED state contains the information for whether the contracts in satlayer is paused or not
/// Take precedence over the PAUSED_CONTRACT_METHOD state
pub(crate) const PAUSED: Item<bool> = Item::new("paused");
