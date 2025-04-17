use cosmwasm_std::{Addr, HexBinary, Uint128};
use cw_storage_plus::Map;

type Service = Addr;
type Earner = Addr;
type Token = String;
type Root = HexBinary;
type PrevRoot = HexBinary;

/// Stores the 2 latest distribution roots for each (service, token) pair
pub const DISTRIBUTION_ROOTS: Map<(&Service, &Token), (PrevRoot, Root)> =
    Map::new("distribution_roots");

/// Stores the live balances of each (service, token) pair
pub const BALANCES: Map<(&Service, &Token), Uint128> = Map::new("balances");

/// Stores the total claimed rewards of each (service, token, earner) pair
pub const CLAIMED_REWARDS: Map<(&Service, &Token, &Earner), Uint128> = Map::new("claimed_rewards");

// Helper functions for distribution roots
pub mod helper {
    use crate::error::RewardsError;
    use crate::state::{PrevRoot, Root, Service, Token, DISTRIBUTION_ROOTS};
    use cosmwasm_std::Storage;

    /// Saves the new distribution root and moves the current root to the previous root
    ///
    /// State is updated as follows: (prev_root, current_root) -> (current_root, root),
    /// prev_root will be dropped.
    pub fn save_distribution_root(
        storage: &mut dyn Storage,
        service: &Service,
        token: &Token,
        root: &Root,
    ) -> Result<(PrevRoot, Root), RewardsError> {
        DISTRIBUTION_ROOTS.update(storage, (service, token), |roots| {
            let (_, current_root) = roots.unwrap_or_default();
            Ok((current_root, root.clone()))
        })
    }

    /// Checks if a root is a current or previous distribution root of the given service and token
    pub fn root_exists(
        storage: &dyn Storage,
        service: &Service,
        token: &Token,
        root: &Root,
    ) -> bool {
        let roots = DISTRIBUTION_ROOTS
            .may_load(storage, (service, token))
            .unwrap_or_default();
        if let Some((prev_root, current_root)) = roots {
            root == &current_root || root == &prev_root
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::Addr;
    use cosmwasm_std::HexBinary;

    #[test]
    fn test_save_distribution_root() {
        let mut deps = mock_dependencies();
        let service = Addr::unchecked("service");
        let token = "token".to_string();
        let first_root =
            HexBinary::from_hex("0ed336b10e9f46e2183d56c888b109aca9533bf0b653ab04e35aa65248f18d29")
                .unwrap();
        let second_root =
            HexBinary::from_hex("0ed336b10e9f46e2183d56c888b109aca9533bf0b653ab04e35aa65248f18d30")
                .unwrap();
        let third_root =
            HexBinary::from_hex("0ed336b10e9f46e2183d56c888b109aca9533bf0b653ab04e35aa65248f18d31")
                .unwrap();

        {
            // Save the distribution root for the first time
            helper::save_distribution_root(&mut deps.storage, &service, &token, &first_root)
                .unwrap();

            // Load the distribution roots
            let roots = DISTRIBUTION_ROOTS
                .load(&deps.storage, (&service, &token))
                .unwrap();
            assert_eq!(roots, (HexBinary::default(), first_root.clone()));
        }
        {
            // Save the distribution root for the second time
            helper::save_distribution_root(&mut deps.storage, &service, &token, &second_root)
                .unwrap();

            // Load the distribution roots
            let roots = DISTRIBUTION_ROOTS
                .load(&deps.storage, (&service, &token))
                .unwrap();
            assert_eq!(roots, (first_root, second_root.clone()));
        }
        {
            // Save the distribution root for the third time
            helper::save_distribution_root(&mut deps.storage, &service, &token, &third_root)
                .unwrap();

            // Load the distribution roots
            let roots = DISTRIBUTION_ROOTS
                .load(&deps.storage, (&service, &token))
                .unwrap();
            assert_eq!(roots, (second_root, third_root));
        }
    }

    #[test]
    fn test_root_exists() {
        let mut deps = mock_dependencies();
        let service = Addr::unchecked("service");
        let token = "token".to_string();

        let prev_root =
            HexBinary::from_hex("0ed336b10e9f46e2183d56c888b109aca9533bf0b653ab04e35aa65248f18d29")
                .unwrap();
        let current_root =
            HexBinary::from_hex("0ed336b10e9f46e2183d56c888b109aca9533bf0b653ab04e35aa65248f18d30")
                .unwrap();

        DISTRIBUTION_ROOTS
            .save(
                &mut deps.storage,
                (&service, &token),
                &(prev_root.clone(), current_root.clone()),
            )
            .unwrap();

        // current_root should exists
        assert!(helper::root_exists(
            &deps.storage,
            &service,
            &token,
            &current_root
        ));

        // prev_root should exists
        assert!(helper::root_exists(
            &deps.storage,
            &service,
            &token,
            &prev_root
        ));

        // a non-existing root should not exists
        let non_existing_root =
            HexBinary::from_hex("0ed336b10e9f46e2183d56c888b109aca9533bf0b653ab04e35aa65248f18d31")
                .unwrap();
        assert!(!helper::root_exists(
            &deps.storage,
            &service,
            &token,
            &non_existing_root
        ));
    }
}
