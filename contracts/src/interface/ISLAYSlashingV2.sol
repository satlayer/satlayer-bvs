// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

/**
 * @title SLAY Slashing V2 Interface
 * @dev Interface for the SLAYSlashingV2 contract.
 * This interface defines the structure and functions for slashing requests in the SatLayer protocol.
 * To be implemented on the SLAYRouter, but separated to allow for separate of slashing related concerns.
 */
interface ISLAYSlashingV2 {
    error LockSlashingNotAuthorized();

    error LockSlashingStatusIsNotPending();

    error LockSlashingExpired();

    error LockSlashingResolutionNotReached();

    /**
     * @dev Emitted when a new slash request is created.
     */
    event SlashingRequested(
        address indexed service, address indexed operator, bytes32 slashId, RequestInfo slashingInfo
    );

    /**
     * @dev Emitted when a slash request has been canceled.
     */
    event SlashingCanceled(
        address indexed service, address indexed operator, bytes32 slashId, RequestInfo slashingInfo
    );

    /**
     * @dev Emitted when a slash request has been locked.
     * This event is emitted when the slashed collateral are moved from the operator's vaults to the router for further processing.
     */
    event SlashingLocked(address indexed service, address indexed operator, bytes32 slashId);

    /// @title Slashing Status
    /// @dev Enum representing the status of a slashing request.
    enum Status {
        /**
         * Earliest stage of a slashing request's lifecycle.
         */
        Pending,
        /**
         * Locked stage is where the slashed collateral are escrow to SatLayer.
         */
        Locked,
        /**
         * Finalized stage is when slashed collateral are moved to destination address.
         */
        Finalized,
        /**
         * Slashing request is canceled when operator as refute adhering to BVS's protocol.
         * Slashing request is also canceled when service has failed to take action beyond expiry.
         */
        Canceled
    }

    /**
     * @title Slashing Request
     * @dev Payload for when a service request slashing of an operator.
     */
    struct Request {
        /**
         * The accused Operator's address.
         */
        address operator;
        /**
         * Collateral amount to be slashed measured in milli bips.
         * Unit is in milli bips. See {ISLAYRegistryV2.SlashParameter} for more details.
         * Cannot be more than service's slashing parameter bounds.
         */
        uint24 mbips;
        /**
         * Timestamp at which alleged misbehaviour occurs.
         */
        uint32 timestamp;
        /**
         * Metadata associated to particular slashing request.
         */
        Metadata metadata;
    }

    /**
     * @title Slashing Metadata
     * @dev Metadata is a struct that contains additional information about the slashing request.
     * Does not affect protocol logic, but can be used for logging or informational purposes.
     */
    struct Metadata {
        string reason;
    }

    /**
     * {RequestInfo} is a struct for internal state tracking.
     * Includes additional data besides the original slashing request payload.
     */
    struct RequestInfo {
        address service;
        Status status;
        Request request;
        uint32 requestTime;
        uint32 requestResolution;
        uint32 requestExpiry;
    }

    /// @dev struct used internally to track locked assets in the router for further processing.
    struct LockedAssets {
        /// The amount of assets locked in the router.
        uint256 amount;
        /// The originating vault address from which the assets were slashed.
        address vault;
    }

    /**
     * @dev Get the current active slashing request for given service operator pair.
     * @param service Address of the service.
     * @param operator Address of the operator.
     */
    function getPendingSlashingRequest(address service, address operator) external view returns (RequestInfo memory);

    /**
     * @dev Get the locked assets for a given slash request.
     * @param slashId The unique identifier for the slash request.
     */
    function getLockedAssets(bytes32 slashId) external view returns (LockedAssets[] memory);

    /**
     * @dev Request slashing from a service to an misbehaving operator.
     * Slashing request for a given operator by the same service can only take place one after another.
     * @param payload {Slashing.RequestPayload}
     */
    function requestSlashing(Request calldata payload) external;

    /**
     * @notice Move all of operator's vaults slashed assets to the router for further processing.
     * It can only be called after the resolution window has passed and before the expiry.
     * @dev Only callable by the service that initiated the slash request.
     * @param slashId The unique identifier for the slash request.
     */
    function lockSlashing(bytes32 slashId) external;
}

/// @title Library for computing slashing request ID
library SlashingRequestId {
    /**
     * Compute the ID by hashing the slashing request data to be used as an identifier within the protocol.
     * The function exclude {RequestStatus} from the hash payload.
     *
     * @param info The slashing request information containing the request and its metadata.
     * @return bytes32 The computed hash of the slashing request.
     */
    function compute(ISLAYSlashingV2.RequestInfo memory info) external pure returns (bytes32) {
        return keccak256(
            abi.encodePacked(
                info.request.operator,
                info.request.mbips,
                info.request.timestamp,
                info.request.metadata.reason,
                info.requestTime,
                info.requestResolution,
                info.requestExpiry,
                info.service
            )
        );
    }
}
