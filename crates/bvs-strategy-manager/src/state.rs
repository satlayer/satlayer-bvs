use crate::ContractError;
use cosmwasm_std::{Addr, Deps, Uint128};
use cw_storage_plus::Map;

pub const STRATEGY_WHITELISTED: Map<&Addr, bool> = Map::new("strategy_whitelisted");

pub fn assert_strategy_whitelisted(deps: Deps, strategy: &Addr) -> Result<(), ContractError> {
    let whitelisted = STRATEGY_WHITELISTED
        .may_load(deps.storage, strategy)?
        .unwrap_or(false);

    if !whitelisted {
        return Err(ContractError::NotWhitelisted {});
    }

    Ok(())
}

pub const STAKER_STRATEGY_SHARES: Map<(&Addr, &Addr), Uint128> = Map::new("staker_strategy_shares");
pub const STAKER_STRATEGY_LIST: Map<&Addr, Vec<Addr>> = Map::new("staker_strategy_list");
pub const MAX_STAKER_STRATEGY_LIST_LENGTH: usize = 10;
