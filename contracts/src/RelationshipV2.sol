// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";

/**
 * @title Relationship Library using Checkpoints
 * @dev This library manages the relationship between a service and an operator.
 * Relationships are tracked using a checkpoint system,
 * allowing for efficient querying of the relationship status at different points in time.
 */
library RelationshipV2 {
    /**
     * @dev Enum representing the registration status between a service and an operator.
     */
    enum Status {
        /**
         * Default state when neither the Operator nor the Service has registered,
         * or when either the Operator or Service has unregistered.
         * The unset value is `uint8(0)` and is used to represent this state, the default value.
         */
        Inactive,
        /**
         * State when both the Operator and Service have registered with each other,
         * indicating a fully established relationship.
         */
        Active,
        /**
         * This state is used when the Operator has registered a Service,
         * but the Service hasn't yet registered,
         * indicating a pending registration from the Service side.
         * This is an Operator-initiated registration, waiting for Service to finalize.
         */
        OperatorRegistered,
        /**
         * This state is used when the Service has registered an Operator,
         * but the Operator hasn't yet registered,
         * indicating a pending registration from the Operator side.
         * This is a Service-initiated registration, waiting for Operator to finalize.
         */
        ServiceRegistered
    }

    /**
     * @title Object of Relationship
     * @dev This struct represents the relationship object that contains the status
     * - 8 bits for the {Status} enum
     * - 32 bits for the slash parameter ID
     * Total: 40 bits (5 bytes) used so far.
     */
    struct Object {
        /// @dev The registration status of the relationship.
        Status status;
        /// @dev The ID of the slash parameter associated with this relationship.
        uint32 slashParameterId;
    }

    /**
     * @dev Hash the service and operator addresses to create a unique key for the `Relationship` mapping.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @return bytes32 The unique key for the service-operator pair.
     */
    function getKey(address service, address operator) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(service, operator));
    }

    /**
     * @dev Pushes a relationship object into a Trace224 checkpoint at the specified timestamp.
     * See Checkpoints.push for more details.
     *
     * IMPORTANT: Never accept `timestamp` as a user input, since an arbitrary `type(uint32).max` key set will disable the
     * library.
     *
     * @param self The Trace224 storage to push into
     * @param timestamp The timestamp to associate with this checkpoint
     * @param obj The relationship object to store
     */
    function push(Checkpoints.Trace224 storage self, uint32 timestamp, Object memory obj) internal {
        uint224 encoded = encode(obj.status, obj.slashParameterId);
        Checkpoints.push(self, timestamp, encoded);
    }

    /**
     * @dev Searches for the relationship object with key lower or equal to the given timestamp.
     * See Checkpoints.upperLookup for more details.
     *
     * @param self The Trace224 storage to search
     * @param timestamp The timestamp to search for
     * @return Object The relationship object at the found checkpoint (or default if not found)
     */
    function upperLookup(Checkpoints.Trace224 storage self, uint32 timestamp) internal view returns (Object memory) {
        uint224 encoded = Checkpoints.upperLookup(self, timestamp);
        return decode(encoded);
    }

    /**
     * @dev Returns the latest relationship object from the checkpoint.
     * See Checkpoints.latest for more details.
     *
     * @param self The Trace224 storage to get the latest value from
     * @return Object The latest relationship object
     */
    function latest(Checkpoints.Trace224 storage self) internal view returns (Object memory) {
        uint224 encoded = Checkpoints.latest(self);
        return decode(encoded);
    }

    /**
     * @dev Encodes the status and slash parameter ID into a single uint224 value.
     * Why encode into uint224, when we could declare a new struct and let Solidity handle it?
     * This is done for efficiency, by packing the Struct into uint224 allowing us to
     * use the existing Checkpoints library which is well audited and optimized for production use.
     *
     * @param status The registration status of the relationship.
     * @param slashParameterId The ID of the slash parameter associated with this relationship.
     * @return uint224 The encoded value containing the status and slash parameter ID.
     */
    function encode(Status status, uint32 slashParameterId) internal pure returns (uint224) {
        uint224 encoded = uint224(uint8(status));
        encoded |= (uint224(slashParameterId) << 8);
        return encoded;
    }

    /**
     * @dev Decodes a uint224 value into an Object struct.
     * IMPORTANT: If the encoded value is `uint224(0)`, it will decode to `Status.Inactive` and `slashParameterId = 0`.
     * This is the default value for the `Object` struct.
     *
     * @param encoded The encoded value containing the status and slash parameter ID.
     * @return obj The decoded Object struct containing the status and slash parameter ID.
     */
    function decode(uint224 encoded) internal pure returns (Object memory) {
        Object memory obj;
        obj.status = Status(uint8(encoded));
        obj.slashParameterId = uint32(encoded >> 8);
        return obj;
    }
}
