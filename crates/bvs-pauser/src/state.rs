use cosmwasm_std::Addr;
use cw_storage_plus::Map;

/// The PAUSED state contains the information for which method are paused on a particular contract
/// Key (composit): (contract_addr, method_msg)
/// Value: u64, the block height at which the method was paused
pub(crate) const PAUSED: Map<(&Addr, &String), u64> = Map::new("paused");
