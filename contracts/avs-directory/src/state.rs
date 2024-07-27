use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult, Storage, Binary};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub enum OperatorAVSRegistrationStatus {
    Registered,
    Unregistered,
}

pub const OWNER: Item<Addr> = Item::new("owner");

pub struct AVSDirectoryStorage {
    pub avs_operator_status: Map<(Addr, Addr), OperatorAVSRegistrationStatus>,
    pub operator_salt_is_spent: Map<(Addr, String), bool>,
    pub delegation_manager: Item<Addr>,
}

impl Default for AVSDirectoryStorage {
    fn default() -> Self {
        AVSDirectoryStorage {
            avs_operator_status: Map::new("avs_operator_status"),
            operator_salt_is_spent: Map::new("operator_salt_is_spent"),
            delegation_manager: Item::new("delegation_manager"),
        }
    }
}

impl AVSDirectoryStorage {
    pub fn save_status(
        &self,
        storage: &mut dyn Storage,
        avs: Addr,
        operator: Addr,
        status: OperatorAVSRegistrationStatus,
    ) -> StdResult<()> {
        self.avs_operator_status.save(storage, (avs, operator), &status)
    }

    pub fn load_status(
        &self,
        storage: &dyn Storage,
        avs: Addr,
        operator: Addr,
    ) -> StdResult<OperatorAVSRegistrationStatus> {
        match self.avs_operator_status.may_load(storage, (avs, operator))? {
            Some(status) => Ok(status),
            None => Ok(OperatorAVSRegistrationStatus::Unregistered),
        }
    }

    pub fn save_salt(&self, storage: &mut dyn Storage, operator: Addr, salt: Binary) -> StdResult<()> {
        self.operator_salt_is_spent.save(storage, (operator, salt.to_base64()), &true)
    }

    pub fn is_salt_spent(&self, storage: &dyn Storage, operator: Addr, salt: Binary) -> StdResult<bool> {
        self.operator_salt_is_spent.may_load(storage, (operator, salt.to_base64())).map(|opt| opt.unwrap_or(false))
    }
}