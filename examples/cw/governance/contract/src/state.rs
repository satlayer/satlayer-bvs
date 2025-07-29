use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub(crate) struct Config {
    pub(crate) registry: Addr,
    pub(crate) router: Addr,
    pub(crate) owner: Addr,
}

/// Config of the contract.
pub(crate) const CONFIG: Item<Config> = Item::new("config");
