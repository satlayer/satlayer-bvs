use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const ROUTER: Item<Addr> = Item::new("router");
pub const REGISTRY: Item<Addr> = Item::new("registry");

/// Contains the code_ids of the contracts that are allowed to be deployed by the factory
/// Permissioned by owner address of factory contract.
/// When operator trigger a deployment of contract, factory contract need to know the code_id of the
/// contract.
/// Which code_id is allowed in the system is determined by the factory contract
pub const CODE_IDS: Map<u64, CodeIdLabel> = Map::new("code_ids");

#[cw_serde]
pub enum CodeIdLabel {
    Cw20Vault,
    BankVault,
    Opaque,
}
