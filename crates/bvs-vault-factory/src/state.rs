use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Key, KeyDeserialize, Map, PrimaryKey};

pub const ROUTER: Item<Addr> = Item::new("router");
pub const REGISTRY: Item<Addr> = Item::new("registry");

/// Contains the code_ids of the contracts that are allowed to be deployed by the factory
/// Permissioned by owner address of factory contract.
/// When operator trigger a deployment of contract, factory contract need to know the code_id of the
/// contract.
/// Which code_id is allowed in the system is determined by the factory contract
pub const CODE_IDS: Map<VaultType, u64> = Map::new("code_ids");

#[cw_serde]
pub enum VaultType {
    Cw20Vault,
    BankVault,
}

impl KeyDeserialize for VaultType {
    type Output = Self;
    const KEY_ELEMS: u16 = 1;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        let trimmed: Vec<u8> = value.into_iter().take_while(|&b| b != 0).collect();
        match std::str::from_utf8(&trimmed) {
            Ok("Cw20Vault") => Ok(VaultType::Cw20Vault),
            Ok("BankVault") => Ok(VaultType::BankVault),
            _ => Err(cosmwasm_std::StdError::generic_err("Invalid VaultType")),
        }
    }
}

impl<'a> PrimaryKey<'a> for VaultType {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        let s = self.to_string();
        let bytes = s.as_bytes();
        let mut buffer = [0u8; 16];
        buffer[..bytes.len().min(16)].copy_from_slice(bytes);
        vec![Key::Val128(buffer)]
    }
}

impl fmt::Display for VaultType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VaultType::Cw20Vault => write!(f, "Cw20Vault"),
            VaultType::BankVault => write!(f, "BankVault"),
        }
    }
}
