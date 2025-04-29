use crate::msg::SlashingRequestPayload;
use bvs_library::addr::{Operator, Service};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_vec, Addr, HexBinary, StdError, StdResult, Storage, Timestamp, Uint64};
use cw_storage_plus::{Item, Key, KeyDeserialize, Map, PrimaryKey};
use sha3::Digest;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// Mapping of vault's Addr to Vault
pub const VAULTS: Map<&Addr, Vault> = Map::new("vaults");

/// Storage for the router address
pub const REGISTRY: Item<Addr> = Item::new("registry");

#[cw_serde]
pub struct Vault {
    pub whitelisted: bool,
}

/// Get the `registry` address
/// If [`instantiate`] has not been called, it will return an [StdError::NotFound]
pub fn get_registry(storage: &dyn Storage) -> StdResult<Addr> {
    REGISTRY
        .may_load(storage)?
        .ok_or(StdError::not_found("registry"))
}

/// Set the `registry` address, called once during `initialization`.
/// The `registry` is the address where the vault calls
pub fn set_registry(storage: &mut dyn Storage, registry: &Addr) -> StdResult<()> {
    REGISTRY.save(storage, registry)?;
    Ok(())
}
/// Store the withdrawal lock period in seconds.
pub const WITHDRAWAL_LOCK_PERIOD: Item<Uint64> = Item::new("withdrawal_lock_period");

/// This is used when the withdrawal lock period is not set.
/// The default value is 7 days.
pub const DEFAULT_WITHDRAWAL_LOCK_PERIOD: Uint64 = Uint64::new(604800);

/// Operator to its managed vaults. Key = (OperatorAddr, VaultAddr)
pub const OPERATOR_VAULTS: Map<(&Addr, &Addr), ()> = Map::new("operator_vaults");

#[cw_serde]
pub struct SlashingRequest {
    /// The core slashing request data including operator, bips, and metadata.
    pub request: SlashingRequestPayload,
    /// The timestamp when the request was submitted.
    pub request_time: Timestamp,
    /// The timestamp after which the request is no longer valid.
    /// This will be `request_time` + `resolution_window` * 2 (as per current slashing parameters)
    pub request_expiry: Timestamp,
}

impl SlashingRequest {
    pub fn new(
        data: SlashingRequestPayload,
        request_time: Timestamp,
        request_expiry: Timestamp,
    ) -> Self {
        Self {
            request: data,
            request_time,
            request_expiry,
        }
    }
}

/// SlashingId stores the id in 256 bit (32 bytes)
#[cw_serde]
pub struct SlashingId(pub HexBinary);

impl fmt::Display for SlashingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl Deref for SlashingId {
    type Target = HexBinary;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SlashingId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HexBinary> for SlashingId {
    fn from(bytes: HexBinary) -> Self {
        Self(bytes)
    }
}

impl From<[u8; 32]> for SlashingId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(HexBinary::from(bytes))
    }
}

impl<'a> PrimaryKey<'a> for SlashingId {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        // Use the HexBinary inside SlashingId to generate the key
        vec![Key::Ref(self.0.as_slice())]
    }
}

impl KeyDeserialize for SlashingId {
    type Output = Self;

    const KEY_ELEMS: u16 = 1;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        // Convert the Vec<u8> to HexBinary, then wrap it in SlashingId
        Ok(SlashingId(HexBinary::from(value)))
    }
}

pub(crate) const SLASHING_ID: Map<(&Service, &Operator), SlashingId> = Map::new("slashing_id");

pub(crate) fn hash_slashing_id(service: &Service, data: &SlashingRequest) -> StdResult<SlashingId> {
    let mut hasher = sha3::Sha3_256::new();
    hasher.update(service.as_bytes());
    hasher.update(to_json_vec(data)?);

    Ok(<[u8; 32]>::from(hasher.finalize()).into())
}

pub(crate) const SLASHING_REQUESTS: Map<SlashingId, SlashingRequest> =
    Map::new("slashing_requests");

pub fn get_active_slashing_requests(
    store: &dyn Storage,
    service: &Service,
    operator: &Operator,
) -> StdResult<Option<SlashingRequest>> {
    // get active slashing_id
    let active_slashing_id = match SLASHING_ID.may_load(store, (service, operator))? {
        Some(id) => id,
        None => return Ok(None),
    };
    // get active slashing from slashing_id
    let active_slashing_request = SLASHING_REQUESTS.may_load(store, active_slashing_id)?;
    match active_slashing_request {
        Some(request) => Ok(Some(request)),
        None => Ok(None),
    }
}

pub fn save_slashing_request(
    store: &mut dyn Storage,
    service: &Service,
    data: &SlashingRequest,
) -> StdResult<SlashingId> {
    // generate slashing_id
    let slashing_id = hash_slashing_id(service, data)?;

    // save slashing id
    SLASHING_ID.save(store, (service, &data.request.operator), &slashing_id)?;

    // save slashing request
    SLASHING_REQUESTS.save(store, slashing_id.clone(), data)?;

    Ok(slashing_id)
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
        let data = SlashingRequestPayload {
            operator: operator.clone(),
            bips: 100,
            timestamp: env.block.time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };
        let slashing_request = SlashingRequest::new(
            data.clone(),
            env.block.time,
            env.block.time.plus_seconds(100),
        );

        let res = save_slashing_request(&mut deps.storage, &service, &slashing_request).unwrap();

        assert_eq!(res, hash_slashing_id(&service, &slashing_request).unwrap());
        assert_eq!(
            res.to_string(),
            "dff7a6f403eff632636533660ab53ab35e7ae0fe2e5dacb160aa7d876a412f09",
            "incorrect hash, hash function may have changed or hash data has changed"
        );

        // assert that SLASHING_ID state is updated
        let slashing_id_res = SLASHING_ID
            .may_load(&deps.storage, (&service, &operator))
            .unwrap();
        assert_eq!(Some(res.clone()), slashing_id_res);

        // assert that SLASHING_REQUESTS state is updated
        let slashing_request_res = SLASHING_REQUESTS.may_load(&deps.storage, res).unwrap();
        assert_eq!(Some(slashing_request), slashing_request_res);
    }
}
