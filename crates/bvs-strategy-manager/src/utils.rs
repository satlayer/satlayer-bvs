use cosmwasm_std::{Addr, Api, StdResult};

pub fn validate_addresses(api: &dyn Api, strategies: &[String]) -> StdResult<Vec<Addr>> {
    strategies
        .iter()
        .map(|addr| api.addr_validate(addr))
        .collect()
}
