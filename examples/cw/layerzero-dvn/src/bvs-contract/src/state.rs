use crate::msg::Packet;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary, Timestamp};
use cw_storage_plus::{Item, Key, Map, Prefixer, PrimaryKey};

#[cw_serde]
pub struct PacketVerificationConfig {
    pub total_active_operators: u64,
    pub required_verifications: u64,
}

#[cw_serde]
pub struct PacketStorage {
    pub packet: Packet,
    pub packet_broadcasted: Timestamp,
    pub packet_verification_finalized: Option<PacketFinalized>,
    pub packet_verification_config: PacketVerificationConfig,
}

/// Key = Packet GUID
/// Value = PacketStorage
pub(crate) const PACKETS: Map<&Vec<u8>, PacketStorage> = Map::new("packets");

#[cw_serde]
pub struct PacketVerified {
    pub payload_hash: HexBinary,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub struct PacketFinalized {
    pub payload_hash: HexBinary,
    pub timestamp: Timestamp,
}

/// Key = (GUID, Operator)
/// Value = PacketVerified
pub(crate) const PACKETS_VERIFIED: Map<(&Vec<u8>, &Addr), PacketVerified> =
    Map::new("packets_hash");

#[cw_serde]
pub(crate) struct Config {
    pub(crate) registry: Addr,
    pub(crate) router: Addr,
    pub(crate) owner: Addr,
    pub(crate) required_verification_threshold: u16, // stores the percentage of required verifications (5000 for 50%)
}

/// Config of the contract.
pub(crate) const CONFIG: Item<Config> = Item::new("config");
