use crate::msg::RequestSlashingPayload;
use bvs_library::addr::{Operator, Service};
use bvs_library::slashing::SlashingRequestId;
use bvs_library::time::DAYS;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_vec, Addr, StdError, StdResult, Storage, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use sha3::Digest;

/// Mapping of vault's Addr to Vault
pub(crate) const VAULTS: Map<&Addr, Vault> = Map::new("vaults");

/// Storage for the router address
pub(crate) const REGISTRY: Item<Addr> = Item::new("registry");

/// Storage for the guardrail contract address
pub(crate) const GUARDRAIL: Item<Addr> = Item::new("guardrail");

#[cw_serde]
pub struct Vault {
    pub whitelisted: bool,
}

/// Get the `registry` address
/// If [`instantiate`] has not been called, it will return an [StdError::NotFound]
pub(crate) fn get_registry(storage: &dyn Storage) -> StdResult<Addr> {
    REGISTRY
        .may_load(storage)?
        .ok_or(StdError::not_found("registry"))
}

/// Set the `registry` address, called once during `initialization`.
/// The `registry` is the address where the vault calls
pub(crate) fn set_registry(storage: &mut dyn Storage, registry: &Addr) -> StdResult<()> {
    REGISTRY.save(storage, registry)?;
    Ok(())
}

/// Store the withdrawal lock period in seconds.
pub(crate) const WITHDRAWAL_LOCK_PERIOD: Item<Uint64> = Item::new("withdrawal_lock_period");

/// This is used when the withdrawal lock period is not set.
/// The default value is 7 days.
pub const DEFAULT_WITHDRAWAL_LOCK_PERIOD: Uint64 = Uint64::new(604800);

/// Operator to its managed vaults. Key = (OperatorAddr, VaultAddr)
pub(crate) const OPERATOR_VAULTS: Map<(&Addr, &Addr), ()> = Map::new("operator_vaults");

/// Stores the expiry window for slashing requests in seconds. Used in `request_expiry`
pub const SLASHING_REQUEST_EXPIRY_WINDOW: Uint64 = Uint64::new(7 * DAYS);

#[cw_serde]
pub struct SlashingRequest {
    /// The core slashing request data including operator, bips, timestamp, and metadata.
    pub request: RequestSlashingPayload,
    /// The timestamp when the request was submitted.
    pub request_time: Timestamp,
    /// The timestamp when the request resolution window will end and becomes eligible for locking.
    /// This will be `request_time` + `resolution_window`.
    pub request_resolution: Timestamp,
    /// The timestamp after which the request is no longer valid.
    /// This will be `request_time` + `resolution_window` + `SLASHING_REQUEST_EXPIRY_WINDOW`
    pub request_expiry: Timestamp,
    /// The status of the slashing request.
    pub status: u8,
    /// The service that initiated the slashing request.
    pub service: Service,
}

#[cw_serde]
#[derive(Copy)]
pub enum SlashingRequestStatus {
    /// The slashing request is pending and has not been processed yet.
    Pending = 0,
    /// The slashing request has been executed and funds are locked.
    Locked = 1,
    /// The slashing request has been finalized.
    Finalized = 2,
    /// The slashing request has been canceled.
    Canceled = 3,
}

impl From<SlashingRequestStatus> for u8 {
    fn from(value: SlashingRequestStatus) -> u8 {
        value as u8
    }
}

impl TryFrom<u8> for SlashingRequestStatus {
    type Error = StdError;

    fn try_from(value: u8) -> Result<Self, StdError> {
        match value {
            0 => Ok(SlashingRequestStatus::Pending),
            1 => Ok(SlashingRequestStatus::Locked),
            2 => Ok(SlashingRequestStatus::Finalized),
            3 => Ok(SlashingRequestStatus::Canceled),
            _ => Err(StdError::generic_err("SlashingRequestStatus out of range")),
        }
    }
}

impl PartialEq<u8> for SlashingRequestStatus {
    fn eq(&self, other: &u8) -> bool {
        *other == u8::from(*self)
    }
}

impl PartialEq<SlashingRequestStatus> for u8 {
    fn eq(&self, other: &SlashingRequestStatus) -> bool {
        *self == u8::from(*other)
    }
}

pub trait SlashingRequestIdHasher {
    /// Generate a slashing request id from the service and slashing request data
    fn hash(data: &SlashingRequest) -> StdResult<SlashingRequestId> {
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(to_json_vec(data)?);

        Ok(<[u8; 32]>::from(hasher.finalize()).into())
    }
}

impl SlashingRequestIdHasher for SlashingRequestId {}

/// Stores the pending slashing request id for a given service and operator.
///
/// Once the slashing request is canceled or finalized,
/// the slashing request id is removed from this map.
pub(crate) const SLASHING_REQUEST_IDS: Map<(&Service, &Operator), SlashingRequestId> =
    Map::new("slashing_request_ids");

/// Stores the slashing request data for a given slashing request id.
///
/// Slashing request won't be removed,
/// hence this map will store all slashing requests.
pub(crate) const SLASHING_REQUESTS: Map<&SlashingRequestId, SlashingRequest> =
    Map::new("slashing_requests");

/// Will return Some() when:
/// 1. Slash is sitting idle in a pending state and beyond the expiry date.
/// 2. Slash is in a pending state and within the expiry date.
/// 3. Slash is locked and not yet finalized, but within the expiry date.
/// 4. Slash is locked and not yet finalized, but beyond the expiry date.
///
/// Will return None when:
/// 1. Slash is canceled
/// 2. Slash is finalized
/// 3. During locking phase the handler catches the slash is expired
pub(crate) fn get_pending_slashing_request(
    store: &dyn Storage,
    service: &Service,
    operator: &Operator,
) -> StdResult<Option<SlashingRequest>> {
    // get pending slashing_id
    let pending_slashing_id = match SLASHING_REQUEST_IDS.may_load(store, (service, operator))? {
        Some(id) => id,
        None => return Ok(None),
    };

    // get pending slashing from slashing_id
    let pending_slashing_request = SLASHING_REQUESTS.may_load(store, &pending_slashing_id)?;
    match pending_slashing_request {
        Some(request) => Ok(Some(request)),
        None => Ok(None),
    }
}

pub(crate) fn save_slashing_request(
    store: &mut dyn Storage,
    service: &Service,
    operator: &Operator,
    data: &SlashingRequest,
) -> StdResult<SlashingRequestId> {
    // generate slashing_id
    let slashing_id = SlashingRequestId::hash(data)?;

    // save slashing id
    SLASHING_REQUEST_IDS.save(store, (service, operator), &slashing_id)?;

    // save slashing request
    SLASHING_REQUESTS.save(store, &slashing_id, data)?;

    Ok(slashing_id)
}

pub(crate) fn update_slashing_request_status(
    store: &mut dyn Storage,
    slashing_request_id: &SlashingRequestId,
    status: SlashingRequestStatus,
) -> StdResult<SlashingRequest> {
    SLASHING_REQUESTS.update(
        store,
        slashing_request_id,
        |slashing_request| match slashing_request {
            Some(mut slashing_request) => {
                slashing_request.status = status.into();
                Ok(slashing_request)
            }
            None => Err(StdError::not_found("Slashing request id not found")),
        },
    )
}

pub(crate) fn remove_slashing_request_id(
    store: &mut dyn Storage,
    service: &Service,
    operator: &Operator,
) -> StdResult<()> {
    SLASHING_REQUEST_IDS.remove(store, (service, operator));
    Ok(())
}

/// Stores the slashed collaterals locked into the router
/// Mapped slash request id and vault address to the absolute amount of that vault
/// The total asset each vault hold may vary such that even if slash bips is the same
/// the absolute number to be slashed translated from bip is varying vault by vault for the same
/// slash request entry.
pub(crate) const SLASH_LOCKED: Map<(SlashingRequestId, &Addr), Uint128> = Map::new("slash_locked");

pub(crate) fn remove_all_slash_locked_by_id(
    store: &mut dyn Storage,
    slashing_request_id: SlashingRequestId,
) -> StdResult<()> {
    SLASH_LOCKED.prefix(slashing_request_id).clear(store, None);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::msg::SlashingMetadata;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    #[test]
    fn test_save_slashing_request() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let service = deps.api.addr_make("service");
        let operator = deps.api.addr_make("operator");
        let data = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: env.block.time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };
        let slashing_request = SlashingRequest {
            request: data.clone(),
            request_time: env.block.time,
            request_resolution: env.block.time.plus_seconds(50),
            request_expiry: env.block.time.plus_seconds(100),
            status: SlashingRequestStatus::Pending.into(),
            service: service.clone(),
        };

        let res = save_slashing_request(&mut deps.storage, &service, &operator, &slashing_request)
            .unwrap();

        assert_eq!(res, SlashingRequestId::hash(&slashing_request).unwrap());
        assert_eq!(
            res.to_string(),
            "4a8ac4cd4fb81675fcdf97b5c25dc6240545016bab314c93cb97461b292fe098",
            "incorrect hash, hash function may have changed or hash data has changed"
        );

        // assert that SLASHING_ID state is updated
        let slashing_id_res = SLASHING_REQUEST_IDS
            .may_load(&deps.storage, (&service, &operator))
            .unwrap();
        assert_eq!(Some(res.clone()), slashing_id_res);

        // assert that SLASHING_REQUESTS state is updated
        let slashing_request_res = SLASHING_REQUESTS.may_load(&deps.storage, &res).unwrap();
        assert_eq!(Some(slashing_request), slashing_request_res);
    }

    #[test]
    fn test_slashing_request_status_partial_eq() {
        // Test SlashingRequestStatus == u8
        assert_eq!(SlashingRequestStatus::Pending, 0u8);
        assert_eq!(SlashingRequestStatus::Locked, 1u8);
        assert_eq!(SlashingRequestStatus::Finalized, 2u8);
        assert_eq!(SlashingRequestStatus::Canceled, 3u8);

        // Test u8 == SlashingRequestStatus
        assert_eq!(0u8, SlashingRequestStatus::Pending);
        assert_eq!(1u8, SlashingRequestStatus::Locked);
        assert_eq!(2u8, SlashingRequestStatus::Finalized);
        assert_eq!(3u8, SlashingRequestStatus::Canceled);

        // Test inequality
        assert_ne!(SlashingRequestStatus::Pending, 1u8);
        assert_ne!(0u8, SlashingRequestStatus::Locked);
    }

    #[test]
    fn test_remove_slashing_request_id() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let service = deps.api.addr_make("service");
        let operator = deps.api.addr_make("operator");
        let data = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: env.block.time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };
        let slashing_request = SlashingRequest {
            request: data.clone(),
            request_time: env.block.time,
            request_resolution: env.block.time.plus_seconds(50),
            request_expiry: env.block.time.plus_seconds(100),
            status: SlashingRequestStatus::Pending.into(),
            service: service.clone(),
        };

        save_slashing_request(&mut deps.storage, &service, &operator, &slashing_request).unwrap();

        let slashing_request_id = SLASHING_REQUEST_IDS
            .may_load(&deps.storage, (&service, &operator))
            .unwrap()
            .unwrap();

        let response = update_slashing_request_status(
            &mut deps.storage,
            &slashing_request_id,
            SlashingRequestStatus::Canceled,
        )
        .unwrap();
        assert_eq!(response.status, SlashingRequestStatus::Canceled);

        let _ = remove_slashing_request_id(&mut deps.storage, &service, &operator);

        assert!(SLASHING_REQUEST_IDS.is_empty(&deps.storage));
    }

    #[test]
    fn test_remove_all_slash_locked_by_id() {
        let mut deps = mock_dependencies();

        // Create a slashing request ID
        let slashing_request_id = SlashingRequestId::from([1u8; 32]);

        // Create multiple vault addresses
        let vault1 = deps.api.addr_make("vault1");
        let vault2 = deps.api.addr_make("vault2");
        let vault3 = deps.api.addr_make("vault3");

        // Add entries to SLASH_LOCKED with the same slashing request ID
        SLASH_LOCKED
            .save(
                &mut deps.storage,
                (slashing_request_id.clone(), &vault1),
                &Uint128::new(100),
            )
            .unwrap();
        SLASH_LOCKED
            .save(
                &mut deps.storage,
                (slashing_request_id.clone(), &vault2),
                &Uint128::new(200),
            )
            .unwrap();
        SLASH_LOCKED
            .save(
                &mut deps.storage,
                (slashing_request_id.clone(), &vault3),
                &Uint128::new(300),
            )
            .unwrap();

        // Create a different slashing request ID to test that it's not affected
        let different_slashing_request_id = SlashingRequestId::from([2u8; 32]);
        SLASH_LOCKED
            .save(
                &mut deps.storage,
                (different_slashing_request_id.clone(), &vault1),
                &Uint128::new(400),
            )
            .unwrap();

        // Verify entries are stored correctly
        assert_eq!(
            SLASH_LOCKED
                .load(&deps.storage, (slashing_request_id.clone(), &vault1))
                .unwrap(),
            Uint128::new(100)
        );
        assert_eq!(
            SLASH_LOCKED
                .load(&deps.storage, (slashing_request_id.clone(), &vault2))
                .unwrap(),
            Uint128::new(200)
        );
        assert_eq!(
            SLASH_LOCKED
                .load(&deps.storage, (slashing_request_id.clone(), &vault3))
                .unwrap(),
            Uint128::new(300)
        );
        assert_eq!(
            SLASH_LOCKED
                .load(
                    &deps.storage,
                    (different_slashing_request_id.clone(), &vault1)
                )
                .unwrap(),
            Uint128::new(400)
        );

        // Call the function to remove all entries with the specific slashing request ID
        remove_all_slash_locked_by_id(&mut deps.storage, slashing_request_id.clone()).unwrap();

        // Verify all entries with the specific ID are removed
        assert!(SLASH_LOCKED
            .may_load(&deps.storage, (slashing_request_id.clone(), &vault1))
            .unwrap()
            .is_none());
        assert!(SLASH_LOCKED
            .may_load(&deps.storage, (slashing_request_id.clone(), &vault2))
            .unwrap()
            .is_none());
        assert!(SLASH_LOCKED
            .may_load(&deps.storage, (slashing_request_id.clone(), &vault3))
            .unwrap()
            .is_none());

        // Verify that entries with different IDs are not affected
        assert_eq!(
            SLASH_LOCKED
                .load(&deps.storage, (different_slashing_request_id, &vault1))
                .unwrap(),
            Uint128::new(400)
        );
    }
}
