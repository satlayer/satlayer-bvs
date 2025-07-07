// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";

/**
 * @title Relationship of Service and Operator
 * @dev This library manages the relationship between a service and an operator,
 * including their registration status.
 * Relationships are tracked using a checkpoint system,
 * allowing for efficient querying of the relationship status at different points in time.
 */
library Relationship {
    /**
     * @dev Enum representing the registration status between a service and an operator.
     * The registration status can be one of the following:
     */
    enum Status {
        /**
         * Default state when neither the Operator nor the Service has registered,
         * or when either the Operator or Service has unregistered.
         * `uint8(0)` is used to represent this state, the default value.
         */
        Inactive,
        /**
         * State when both the Operator and Service have registered with each other,
         * indicating a fully established relationship.
         */
        Active,
        /**
         * This state is used when the Operator has registered an Service,
         * but the Service hasn't yet registered,
         * indicating a pending registration from the Service side.
         * This is Operator-initiated registration, waiting for Service to finalize.
         */
        OperatorRegistered,
        /**
         * This state is used when the Service has registered an Operator,
         * but the Operator hasn't yet registered,
         * indicating a pending registration from the Operator side.
         * This is Service-initiated registration, waiting for Operator to finalize.
         */
        ServiceRegistered
    }

    struct Object {
        Status status;
        uint32 slashingId;
    }

    /**
     * @dev Hash the service and operator addresses to create a unique key for the `Relationship` map.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @return bytes32 The unique key for the service-operator pair.
     */
    function getKey(address service, address operator) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(service, operator));
    }

    /// @dev see Checkpoints.push
    function push(Checkpoints.Trace224 storage self, uint32 timestamp, Object memory obj) internal {
        uint224 encoded = encode(obj.status, obj.slashingId);
        Checkpoints.push(self, timestamp, encoded);
    }

    /// @dev see Checkpoints.pushRecent
    function lowerLookup(Checkpoints.Trace224 storage self, uint32 timestamp) internal view returns (Object memory) {
        uint224 encoded = Checkpoints.lowerLookup(self, timestamp);
        return decode(encoded);
    }

    /// @dev see Checkpoints.lowerLookupRecent
    function upperLookup(Checkpoints.Trace224 storage self, uint32 timestamp) internal view returns (Object memory) {
        uint224 encoded = Checkpoints.upperLookup(self, timestamp);
        return decode(encoded);
    }

    /// @dev see Checkpoints.lowerLookupRecent
    function upperLookupRecent(Checkpoints.Trace224 storage self, uint32 timestamp)
        internal
        view
        returns (Object memory)
    {
        uint224 encoded = Checkpoints.upperLookupRecent(self, timestamp);
        return decode(encoded);
    }

    /// @dev see Checkpoints.latest
    function latest(Checkpoints.Trace224 storage self) internal view returns (Object memory) {
        uint224 encoded = Checkpoints.latest(self);
        return decode(encoded);
    }

    /// @dev see Checkpoints.latestCheckpoint
    function latestCheckpoint(Checkpoints.Trace224 storage self)
        internal
        view
        returns (bool exists, uint32 timestamp, Object memory obj)
    {
        (bool exists, uint32 key, uint224 encoded) = Checkpoints.latestCheckpoint(self);
        return (exists, key, decode(encoded));
    }

    /// @dev see Checkpoints.length
    function length(Checkpoints.Trace224 storage self) internal view returns (uint256) {
        return Checkpoints.length(self);
    }

    function encode(Status status, uint32 slashingId) internal pure returns (uint224) {
        uint224 encoded = uint224(uint8(status));
        encoded |= (uint224(slashingId) << 8);
        return encoded;
    }

    function decode(uint224 encoded) internal pure returns (Object memory) {
        Object memory obj;
        obj.status = Status(uint8(encoded));
        obj.slashingId = uint32(encoded >> 8);
        return obj;
    }
}
