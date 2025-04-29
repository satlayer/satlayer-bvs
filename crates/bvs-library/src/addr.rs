use cosmwasm_std::{Addr, Api, StdResult};

pub type Service = Addr;
pub type Operator = Addr;

/// Validate a list of addresses
/// Return a vector of validated addresses
pub fn validate_addrs(api: &dyn Api, addr: &[String]) -> StdResult<Vec<Addr>> {
    addr.iter().map(|addr| api.addr_validate(addr)).collect()
}
