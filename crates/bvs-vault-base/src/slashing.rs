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
    let slashable = SLASHABLE.may_load(store)?.unwrap_or(false);

    Ok(slashable)
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

    use super::*;
    use cosmwasm_std::testing::MockStorage;

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
