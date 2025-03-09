use cosmwasm_std::{Addr, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::Map;

/// Mapping of staker to their shares in the vault
const SHARES: Map<&Addr, Uint128> = Map::new("shares");

/// Unchecked add, you can add zero shares—accounting module won't check this.
/// Adding zero shares is as good as not running this function.
pub fn add_shares(
    storage: &mut dyn Storage,
    recipient: &Addr,
    new_shares: Uint128,
) -> Result<Uint128, StdError> {
    SHARES.update(storage, recipient, |recipient_shares| -> StdResult<_> {
        recipient_shares
            .unwrap_or(Uint128::zero())
            .checked_add(new_shares)
            .map_err(StdError::from)
    })
}

/// Unchecked sub, you can remove zero shares—accounting module won't check this.
/// Removing zero shares is as good as not running this function.
pub fn sub_shares(
    storage: &mut dyn Storage,
    recipient: &Addr,
    shares: Uint128,
) -> Result<Uint128, StdError> {
    SHARES.update(storage, recipient, |recipient_shares| -> StdResult<_> {
        recipient_shares
            .unwrap_or(Uint128::zero())
            .checked_sub(shares)
            .map_err(StdError::from)
    })
}

/// Get the shares of a staker, returns zero if not found
pub fn get_shares(storage: &dyn Storage, staker: &Addr) -> StdResult<Uint128> {
    SHARES
        .may_load(storage, staker)
        .map(|res| res.unwrap_or(Uint128::zero()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::Addr;

    #[test]
    fn get_zero_shares() {
        let store = MockStorage::new();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());
    }

    #[test]
    fn add_and_get_shares() {
        let mut store = MockStorage::new();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());

        let new_shares = Uint128::new(12345);
        add_shares(&mut store, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, new_shares);

        add_shares(&mut store, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::new(24690));
    }

    #[test]
    fn add_and_sub_shares() {
        let mut store = MockStorage::new();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());

        let new_shares = Uint128::new(12345);
        add_shares(&mut store, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, new_shares);

        let remove_shares = Uint128::new(1234);
        sub_shares(&mut store, &staker, remove_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::new(11_111));
    }
}
