use crate::msg::VaultType;
use crate::ContractError;
use cosmwasm_std::{Addr, StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};

pub(crate) const ROUTER: Item<Addr> = Item::new("router");
pub(crate) const REGISTRY: Item<Addr> = Item::new("registry");

/// Contains the code_ids of the contracts that are allowed to be deployed by the factory.
/// > Permissioned by owner address of factory contract.
/// > When an operator triggers a deployment of a contract,
/// > the factory contract needs to know the code_id of the contract.
const CODE_IDS: Map<&u8, u64> = Map::new("code_ids");

pub(crate) fn get_code_id(
    store: &dyn Storage,
    vault_type: &VaultType,
) -> Result<u64, ContractError> {
    CODE_IDS
        .load(store, &vault_type.into())
        .map_err(|_| ContractError::CodeIdNotFound {})
}

pub(crate) fn set_code_id(
    store: &mut dyn Storage,
    vault_type: &VaultType,
    code_id: &u64,
) -> StdResult<()> {
    CODE_IDS.save(store, &vault_type.into(), code_id)
}

impl From<&VaultType> for u8 {
    fn from(value: &VaultType) -> u8 {
        match value {
            VaultType::Bank => 1,
            VaultType::Cw20 => 2,
            VaultType::BankTokenized => 3,
            VaultType::Cw20Tokenized => 4,
        }
    }
}

impl TryFrom<u8> for VaultType {
    type Error = StdError;

    fn try_from(value: u8) -> Result<Self, StdError> {
        match value {
            1 => Ok(VaultType::Bank),
            2 => Ok(VaultType::Cw20),
            3 => Ok(VaultType::BankTokenized),
            4 => Ok(VaultType::Cw20Tokenized),
            _ => Err(StdError::generic_err("VaultType out of range")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

    #[test]
    fn into_u8_value() {
        let value: u8 = (&VaultType::Bank).into();
        assert_eq!(value, 1);

        let value: u8 = (&VaultType::Cw20).into();
        assert_eq!(value, 2);

        let value: u8 = (&VaultType::BankTokenized).into();
        assert_eq!(value, 3);

        let value: u8 = (&VaultType::Cw20Tokenized).into();
        assert_eq!(value, 4);
    }

    #[test]
    fn set_get_test_code_id() {
        let mut store = MockStorage::new();
        let vault_type = VaultType::Bank;
        let code_id = 1234;

        set_code_id(&mut store, &vault_type, &code_id).unwrap();
        let res = get_code_id(&store, &vault_type).unwrap();
        assert_eq!(res, code_id);

        let res = get_code_id(&store, &VaultType::Cw20);
        assert!(res.is_err());

        let res = get_code_id(&store, &VaultType::Bank).unwrap();
        assert_eq!(res, code_id);

        let res = get_code_id(&store, &VaultType::BankTokenized);
        assert!(res.is_err());
        set_code_id(&mut store, &VaultType::BankTokenized, &code_id).unwrap();
        let res = get_code_id(&store, &VaultType::BankTokenized).unwrap();
        assert_eq!(res, code_id);

        let res = get_code_id(&store, &VaultType::Cw20Tokenized);
        assert!(res.is_err());
        set_code_id(&mut store, &VaultType::Cw20Tokenized, &code_id).unwrap();
        let res = get_code_id(&store, &VaultType::Cw20Tokenized).unwrap();
        assert_eq!(res, code_id);
    }
}
