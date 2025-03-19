use cosmwasm_std::{Addr, Env, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{SnapshotMap, Strategy};

/// Mapping of staker to their shares in the vault
// const SHARES: Map<&Addr, Uint128> = Map::new("shares");
const SHARES: SnapshotMap<&Addr, Uint128> = SnapshotMap::new(
    "shares",
    "shares-checkpoints",
    "shares-changelog",
    Strategy::EveryBlock,
);

/// Unchecked add, you can add zero shares—accounting module won't check this.
/// Adding zero shares is as good as not running this function.
pub fn add_shares(
    storage: &mut dyn Storage,
    env: &Env,
    recipient: &Addr,
    new_shares: Uint128,
) -> Result<Uint128, StdError> {
    SHARES.update(
        storage,
        recipient,
        env.block.height,
        |recipient_shares| -> StdResult<_> {
            recipient_shares
                .unwrap_or(Uint128::zero())
                .checked_add(new_shares)
                .map_err(StdError::from)
        },
    )
}

/// Unchecked sub, you can remove zero shares—accounting module won't check this.
/// Removing zero shares is as good as not running this function.
pub fn sub_shares(
    storage: &mut dyn Storage,
    env: &Env,
    recipient: &Addr,
    shares: Uint128,
) -> Result<Uint128, StdError> {
    SHARES.update(
        storage,
        recipient,
        env.block.height,
        |recipient_shares| -> StdResult<_> {
            recipient_shares
                .unwrap_or(Uint128::zero())
                .checked_sub(shares)
                .map_err(StdError::from)
        },
    )
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
    use cosmwasm_std::testing::{mock_env, MockStorage};
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
        let env = mock_env();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());

        let new_shares = Uint128::new(12345);
        add_shares(&mut store, &env, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, new_shares);

        add_shares(&mut store, &env, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::new(24690));
    }

    #[test]
    fn add_and_sub_shares() {
        let mut store = MockStorage::new();
        let env = mock_env();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());

        let new_shares = Uint128::new(12345);
        add_shares(&mut store, &env, &staker, new_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, new_shares);

        let remove_shares = Uint128::new(1234);
        sub_shares(&mut store, &env, &staker, remove_shares).unwrap();
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::new(11_111));
    }

    #[test]
    fn load_shares_at_diff_blockheight() {
        let mut store = MockStorage::new();
        let mut env = mock_env();
        let staker = Addr::unchecked("staker");
        let shares = get_shares(&store, &staker).unwrap();
        assert_eq!(shares, Uint128::zero());

        // store share at current blockheight
        SHARES
            .save(&mut store, &staker, &Uint128::new(100), env.block.height)
            .expect("failed to save shares");

        // load latest share
        let shares = SHARES
            .may_load(&store, &staker)
            .expect("failed to load shares");
        assert_eq!(shares, Some(Uint128::new(100)));

        // load shares at current blockheight
        let shares = SHARES
            .may_load_at_height(&store, &staker, env.block.height)
            .expect("failed to load shares");
        assert_eq!(shares, None);

        // load shares at current blockheight + 1
        let shares = SHARES
            .may_load_at_height(&store, &staker, env.block.height + 1)
            .expect("failed to load shares");
        assert_eq!(shares, Some(Uint128::new(100)));

        // advance block by 10
        env.block.height += 10;

        // update block by adding 99 shares
        SHARES
            .update(&mut store, &staker, env.block.height, |shares| {
                shares
                    .unwrap_or(Uint128::zero())
                    .checked_add(Uint128::new(99))
                    .map_err(StdError::from)
            })
            .expect("failed to update shares");

        // load latest share
        let shares = SHARES
            .may_load(&store, &staker)
            .expect("failed to load shares");
        assert_eq!(shares, Some(Uint128::new(199)));

        // load shares at current blockheight
        let shares = SHARES
            .may_load_at_height(&store, &staker, env.block.height)
            .expect("failed to load shares");
        assert_eq!(shares, Some(Uint128::new(100)));

        // load shares at current blockheight + 1
        let shares = SHARES
            .may_load_at_height(&store, &staker, env.block.height + 1)
            .expect("failed to load shares");
        assert_eq!(shares, Some(Uint128::new(199)));

        // load share at previous blockheight
        let shares = SHARES
            .may_load_at_height(&store, &staker, env.block.height - 10 + 1)
            .expect("failed to load shares");
        assert_eq!(shares, Some(Uint128::new(100)));
    }
}
