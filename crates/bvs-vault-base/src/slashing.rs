use crate::error::VaultError;
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;

const SLASHABLE: Item<bool> = Item::new("slashable");

pub fn set_slashable(store: &mut dyn Storage, slashable: bool) -> Result<(), VaultError> {
    // Set the vault as slashable
    SLASHABLE.save(store, &slashable)?;

    Ok(())
}

pub fn get_slashable(store: &dyn Storage) -> StdResult<bool> {
    match SLASHABLE.may_load(store)? {
        // The vault is slashable
        Some(true) => Ok(true),

        // The vault is not slashable
        Some(false) => Ok(false),

        // For some reason the vault do not have slashability flag set.
        // Then effectively it is not slashable.
        None => Ok(false),
    }
}

pub fn assert_slashable(store: &dyn Storage) -> Result<(), VaultError> {
    match SLASHABLE.may_load(store)?.ok_or(VaultError::unauthorized(
        "The vault do not have slashability flag",
    )) {
        // The vault is slashable
        Ok(true) => Ok(()),

        // The vault is not slashable
        Ok(false) => Err(VaultError::unauthorized("The vault is not slashable")),

        // For some reason the vault do not have slashability flag set.
        Err(_) => Err(VaultError::unauthorized(
            "The vault do not have slashability flag",
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::router::set_router;

    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, MockStorage},
        MessageInfo,
    };
    use cw_multi_test::App;

    #[test]
    fn test_assert_slashable() {
        let mut store = MockStorage::new();

        // Test that the vault is not slashable
        assert!(assert_slashable(&store).is_err());

        // Set the vault as slashable
        SLASHABLE.save(&mut store, &true).unwrap();

        // Test that the vault is slashable
        assert!(assert_slashable(&store).is_ok());

        // Set the vault as not slashable
        SLASHABLE.save(&mut store, &false).unwrap();

        // Test that the vault is not slashable
        assert!(assert_slashable(&store).is_err());
    }
}
