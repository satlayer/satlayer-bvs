// SPDX-License-Identifier: MIT
pragma solidity ^0.8.10;

interface IBVS {
    event PacketBroadcasted(
        bytes32 indexed guid, uint32 indexed srcEid, uint32 indexed dstEid, uint256 requiredVerificationCount
    );

    event PacketSubmitted(bytes32 indexed guid, address indexed operator, bytes32 payloadHash);

    event PacketFinalized(bytes32 indexed guid, bytes32 payloadHash);

    struct Packet {
        uint64 nonce; // Unique identifier for the packet
        uint32 srcEid; // Source chain's endpoint ID
        uint32 dstEid; // Destination chain's endpoint ID
        bytes32 sender; // Address of the sender
        bytes32 receiver; // Address of the receiver
        bytes32 guid;
        bytes payload; // Data being sent in the packet
    }

    struct PacketStorage {
        Packet packet; // The packet data
        uint32 broadcastedAt; // Timestamp of when the packet was broadcasted in seconds
        uint8 requiredVerificationCount; // Number of verifications required for this packet
        bytes32 finalizedPayloadHash; // Hash of the finalized payload
        uint32 finalizedAt; // Timestamp of when the packet was finalized in seconds
    }

    function broadcastPacket(Packet calldata packet) external virtual;

    function submitPacket(bytes32 packetGuid, bytes32 payloadHash) external virtual;

    function getPacket(bytes32 packetGuid) external view virtual returns (PacketStorage memory);

    function finalizePacketVerification(bytes32 packetGuid, bytes32 payloadHash) external virtual;

    function getFinalizedPayloadHash(bytes32 packetGuid) external view virtual returns (bytes32);

    function registerOperator(address operator) external virtual;

    function enableSlashing() external virtual;
}
