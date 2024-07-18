use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Map;
use std::collections::HashSet;

#[cw_serde]
pub enum OperatorStatus {
    Registered,
    Unregistered,
}

#[cw_serde]
pub struct OperatorRegistration {
    pub operator: String,
    pub status: OperatorStatus,
    pub salt: String,
}

pub struct AVSDirectoryStorage {
    pub operator_status: Map<String, OperatorStatus>,
    pub salt_spent: HashSet<String>,
}

impl Default for AVSDirectoryStorage {
    fn default() -> Self {
        AVSDirectoryStorage {
            operator_status: Map::new("operator_status"),
            salt_spent: HashSet::new(),
        }
    }
}

impl AVSDirectoryStorage {
    pub fn save(&self, storage: &mut dyn Storage, operator: String, status: OperatorStatus) -> StdResult<()> {
        self.operator_status.save(storage, operator, &status)?;
        Ok(())
    }

    pub fn load(storage: &dyn Storage, operator: String) -> StdResult<OperatorStatus> {
        let operator_status = Map::new("operator_status").load(storage, operator)?;
        Ok(operator_status)
    }
}
