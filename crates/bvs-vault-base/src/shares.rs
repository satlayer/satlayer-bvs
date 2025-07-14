use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::Map;

/// Mapping of staker to their shares in the vault
const SHARES: Map<&Addr, Uint128> = Map::new("shares");

/// Mapping of controller to their queued withdrawal associated with unlock timestamp in the vault
const QUEUED_WITHDRAWAL: Map<&Addr, QueuedWithdrawalInfo> = Map::new("queued_withdrawal");

#[cw_serde]
pub struct QueuedWithdrawalInfo {
    pub queued_shares: Uint128,
    pub unlock_timestamp: Timestamp,
}

impl Default for QueuedWithdrawalInfo {
    fn default() -> Self {
        Self {
            queued_shares: Uint128::zero(),
            unlock_timestamp: Timestamp::from_seconds(0),
        }
    }
}

/// Add shares to a staker, returns the updated shares
///
/// This function doesn't check if `new_shares` is zero
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

/// Subtract shares from a staker, returns the updated shares
///
/// This function doesn't check if `shares` is zero
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

/// Update queued withdrawal info with a new unlock timestamp for a receipent
pub fn update_queued_withdrawal_info(
    storage: &mut dyn Storage,
    controller: &Addr,
    queued_withdrawal_info: QueuedWithdrawalInfo,
) -> StdResult<QueuedWithdrawalInfo> {
    QUEUED_WITHDRAWAL.update(storage, controller, |existing| -> StdResult<_> {
        match existing {
            Some(old) => {
                let new_queued_shares = old
                    .queued_shares
                    .checked_add(queued_withdrawal_info.queued_shares)
                    .map_err(StdError::from)?;

                Ok(QueuedWithdrawalInfo {
                    queued_shares: new_queued_shares,
                    unlock_timestamp: queued_withdrawal_info.unlock_timestamp,
                })
            }
            None => Ok(queued_withdrawal_info),
        }
    })
}

/// When staker redeems queued withdrawal, remove the data of staker
pub fn remove_queued_withdrawal_info(storage: &mut dyn Storage, controller: &Addr) {
    QUEUED_WITHDRAWAL.remove(storage, controller)
}

/// Get the queued withdrawal info and lock time for a controller.
pub fn get_queued_withdrawal_info(
    storage: &dyn Storage,
    controller: &Addr,
) -> StdResult<QueuedWithdrawalInfo> {
    QUEUED_WITHDRAWAL
        .may_load(storage, controller)
        .map(|res| res.unwrap_or_default())
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

    #[test]
    fn set_and_get_queued_withdrawal_info() {
        let mut store = MockStorage::new();
        let staker = Addr::unchecked("staker");

        let result = get_queued_withdrawal_info(&mut store, &staker).unwrap();
        assert_eq!(result.queued_shares, Uint128::zero());
        assert_eq!(result.unlock_timestamp, Timestamp::from_seconds(0));

        let queued_withdrawal_info1 = QueuedWithdrawalInfo {
            queued_shares: Uint128::new(123),
            unlock_timestamp: Timestamp::from_seconds(0),
        };

        let result =
            update_queued_withdrawal_info(&mut store, &staker, queued_withdrawal_info1.clone())
                .unwrap();
        assert_eq!(result.queued_shares, queued_withdrawal_info1.queued_shares);
        assert_eq!(
            result.unlock_timestamp,
            queued_withdrawal_info1.unlock_timestamp
        );

        let queued_withdrawal_info2 = QueuedWithdrawalInfo {
            queued_shares: Uint128::new(456),
            unlock_timestamp: Timestamp::from_seconds(0),
        };

        let result =
            update_queued_withdrawal_info(&mut store, &staker, queued_withdrawal_info2.clone())
                .unwrap();
        assert_eq!(result.queued_shares, Uint128::new(579));
        assert_eq!(
            result.unlock_timestamp,
            queued_withdrawal_info2.unlock_timestamp
        );

        let result = get_queued_withdrawal_info(&mut store, &staker).unwrap();
        assert_eq!(result.queued_shares, Uint128::new(579));
        assert_eq!(
            result.unlock_timestamp,
            queued_withdrawal_info2.unlock_timestamp
        );

        remove_queued_withdrawal_info(&mut store, &staker);

        let result = get_queued_withdrawal_info(&mut store, &staker).unwrap();
        assert_eq!(result.queued_shares, Uint128::zero());
        assert_eq!(result.unlock_timestamp, Timestamp::from_seconds(0));
    }
}
