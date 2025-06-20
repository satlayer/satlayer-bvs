use crate::state::{PacketStorage, PacketVerified};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, HexBinary, Uint64};

/// Instantiate message for the contract.
#[cw_serde]
pub struct InstantiateMsg {
    pub registry: String,
    pub router: String,
    /// Used for administrative operations.
    pub owner: String,
    pub required_verification_threshold: u16, // stores the percentage of required verifications (5000 for 50%)
}

/// Migration message for the contract.
/// > This is used when the contract is migrated to a new version.
/// > You can keep this empty if you don't need to do anything special during migration.
#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct Packet {
    /// The nonce of the message in the pathway
    pub nonce: Uint64,
    /// The source endpoint ID
    pub src_eid: u32,
    /// The sender address (20 bytes)
    pub sender: HexBinary,
    /// The destination endpoint ID
    pub dst_eid: u32,
    /// The receiving address (32 bytes)
    pub receiver: HexBinary,
    /// A global unique identifier (32 bytes)
    pub guid: HexBinary,
    /// The message payload
    pub message: HexBinary,
}

/// Execute messages for the contract.
/// These messages allow modifying the state of the contract and emits event.
#[cw_serde]
pub enum ExecuteMsg {
    /// Broadcast a new packet for operators to verify
    BroadcastPacket(Packet),
    /// Operator calls this to post their packet verification.
    SubmitPacket {
        packet_guid: HexBinary,
        payload_hash: HexBinary,
    },
    /// Check if quorum has been reached and select the payload hash
    FinalizePacketVerification { packet_guid: HexBinary },
    /// Resolve the slashing process for the `operator` by canceling it.
    SlashingCancel { operator: String },
    /// Move to the 2nd stage of the slashing process, locking the operator vault's funds.
    SlashingLock { operator: String },
    /// Finalize the slashing process for the `operator`.
    SlashingFinalize { operator: String },
    /// Register a new operator for the squaring service.
    RegisterOperator { operator: String },
    /// Deregister an operator from running the squaring service.
    DeregisterOperator { operator: String },
    /// Enable slashing for the squaring service.
    EnableSlashing {},
    /// Disable slashing for the squaring service.
    DisableSlashing {},
}

/// Query messages for the contract.
/// These messages allow querying the state of the contract.
/// Does not allow modifying the state of the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get response for a given `input` with `operator` that responded to the request.
    #[returns(GetPacketResponse)]
    GetPacket(HexBinary),

    #[returns(GetPacketSubmissionResponse)]
    GetPacketSubmission {
        packet_guid: HexBinary,
        operator: String,
    },
}

/// Response for the `GetPacket` query.
#[cw_serde]
pub struct GetPacketResponse(PacketStorage);

/// Response for the `GetPacketSubmission` query.
#[cw_serde]
pub struct GetPacketSubmissionResponse(PacketVerified);
