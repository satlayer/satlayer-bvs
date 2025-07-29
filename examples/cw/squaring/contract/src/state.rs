use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Key = Input
/// Value = Requester of the computation
pub(crate) const REQUESTS: Map<&i64, Addr> = Map::new("requests");

/// Key = (Input, Operator)
/// Value = Output
/// Each Operator writes their own response to the output.
pub(crate) const RESPONSES: Map<(i64, &Addr), i64> = Map::new("responses");

#[cw_serde]
pub(crate) struct Config {
    pub(crate) registry: Addr,
    pub(crate) router: Addr,
    pub(crate) owner: Addr,
}

/// Config of the contract.
pub(crate) const CONFIG: Item<Config> = Item::new("config");
