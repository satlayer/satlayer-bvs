// SPDX-License-Identifier: MIT
pragma solidity ^0.8.10;

import {IBVS} from "./interface/IBVS.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {EnumerableMap} from "@openzeppelin/contracts/utils/structs/EnumerableMap.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {ISLAYRegistryV2} from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";
import {RelationshipV2} from "@satlayer/contracts/RelationshipV2.sol";

contract BVS is Ownable, IBVS {
    using EnumerableMap for EnumerableMap.AddressToBytes32Map;

    address private _SLAYRegistry;

    address private _SLAYRouter;

    /// @dev Stores the bips required for verification to be finalized.
    uint16 private requiredVerificationThreshold;

    /// @dev Mapping to store packets by their GUID.
    mapping(bytes32 guid => PacketStorage) private packets;

    /// @dev Mapping to track the payload hashes submitted by operators for each packet.
    mapping(bytes32 guid => mapping(address operator => bytes32 payloadHash)) private packetsVerified;

    mapping(bytes32 guid => mapping(bytes32 payloadHash => uint8 count)) private payloadHashCounts;

    constructor(address slayRegistry, address slayRouter, address owner, uint16 verificationThreshold) Ownable(owner) {
        _SLAYRegistry = slayRegistry;
        _SLAYRouter = slayRouter;
        requiredVerificationThreshold = verificationThreshold;

        // register the BVS service in the SLAY registry
        ISLAYRegistryV2(_SLAYRegistry).registerAsService(
            "www.bvs.com", "BVS+DVN"
        );
    }

    /// @inheritdoc IBVS
    function broadcastPacket(Packet calldata packet) external override onlyOwner {
        // Ensure the packet GUID does not already exist
        require(packets[packet.guid].packet.guid == bytes32(0), "Packet already exists");

        uint256 totalActiveOperators = ISLAYRegistryV2(_SLAYRegistry).getActiveOperatorCount(address(this));

        uint256 requiredVerificationCount =
            Math.mulDiv(totalActiveOperators, requiredVerificationThreshold, 10_000, Math.Rounding.Ceil);

        require(requiredVerificationCount > 0, "Require at least one active operator to verify");

        // Create a new PacketStorage instance and store it
        packets[packet.guid] = PacketStorage({
            packet: packet,
            requiredVerificationCount: uint8(requiredVerificationCount), // safe to cast since max active operators is capped to 5
            broadcastedAt: uint32(block.timestamp),
            finalizedPayloadHash: bytes32(0),
            finalizedAt: 0
        });

        // Emit an event for the broadcasted packet
        emit PacketBroadcasted(packet.guid, packet.srcEid, packet.dstEid, requiredVerificationCount);
    }

    /// @inheritdoc IBVS
    function submitPacket(bytes32 packetGuid, bytes32 payloadHash) external override {
        // Ensure the packet exists
        require(packetGuid != bytes32(0), "Invalid packet GUID");
        require(packets[packetGuid].packet.guid != bytes32(0), "Packet does not exist");

        // ensure operator has not already submitted a payload hash for this packet
        require(packetsVerified[packetGuid][msg.sender] == bytes32(0), "Operator has already submitted a payload hash");

        PacketStorage storage packetStorage = packets[packetGuid];

        // ensure the operator is actively registered to the service
        require(
            ISLAYRegistryV2(_SLAYRegistry).getRelationshipStatusAt(
                address(this), msg.sender, packetStorage.broadcastedAt
            ) == RelationshipV2.Status.Active,
            "Operator was not actively registered at the time of packet broadcast"
        );

        // save the payload hash for the operator
        packetsVerified[packetGuid][msg.sender] = payloadHash;
        // increment the count of unique payload hashes for this packet
        payloadHashCounts[packetGuid][payloadHash] += 1;

        emit PacketSubmitted(packetGuid, msg.sender, payloadHash);
    }

    function getPacket(bytes32 packetGuid) external view override returns (PacketStorage memory) {
        // Ensure the packet exists
        require(packetGuid != bytes32(0), "Invalid packet GUID");
        require(packets[packetGuid].packet.guid != bytes32(0), "Packet does not exist");

        // Return the packet storage for the given GUID
        return packets[packetGuid];
    }

    /// @inheritdoc IBVS
    function finalizePacketVerification(bytes32 packetGuid, bytes32 payloadHash) external override onlyOwner {
        // Ensure the packet exists
        require(packetGuid != bytes32(0), "Invalid packet GUID");
        require(packets[packetGuid].packet.guid != bytes32(0), "Packet does not exist");

        PacketStorage storage packetStorage = packets[packetGuid];

        // Ensure the packet has not already been finalized
        require(packetStorage.finalizedPayloadHash == bytes32(0), "Packet already finalized");

        // Ensure the number of unique payload hashes meets the required verification count
        require(
            payloadHashCounts[packetGuid][payloadHash] >= packetStorage.requiredVerificationCount,
            "Not enough verifications"
        );

        // Finalize the packet verification
        packetStorage.finalizedPayloadHash = payloadHash;
        packetStorage.finalizedAt = uint32(block.timestamp);

        // Emit an event for the finalized packet
        emit PacketFinalized(packetGuid, payloadHash);
    }

    /// @inheritdoc IBVS
    function getFinalizedPayloadHash(bytes32 packetGuid) external view override returns (bytes32) {
        // Ensure the packet exists
        require(packetGuid != bytes32(0), "Invalid packet GUID");
        require(packets[packetGuid].packet.guid != bytes32(0), "Packet does not exist");

        // Return the finalized payload hash for the packet
        return packets[packetGuid].finalizedPayloadHash;
    }

    /// @inheritdoc IBVS
    function registerOperator(address operator) external override onlyOwner {
        // Register the operator in the SLAY registry
        ISLAYRegistryV2(_SLAYRegistry).registerOperatorToService(operator);
    }

    /// @inheritdoc IBVS
    function enableSlashing() external override onlyOwner {
        // Enable slashing for the operator in the SLAY registry
        ISLAYRegistryV2(_SLAYRegistry).enableSlashing(
            ISLAYRegistryV2.SlashParameter({
                destination: address(this),
                maxMbips: 50 * 100_000, // 50% max slashing
                resolutionWindow: 7 days
            })
        );
    }
}
