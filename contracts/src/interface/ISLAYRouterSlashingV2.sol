// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

/**
 * @title SLAY Router Slashing V2 Interface
 * @dev Interface for the Slashing.
 * This interface defines the structure and functions for slashing requests in the SatLayer protocol.
 * To be implemented on the SLAYRouter, but separated to allow for separation of slashing related concerns.
 */
interface ISLAYRouterSlashingV2 {
    /*//////////////////////////////////////////////////////////////
                                ERRORS
    //////////////////////////////////////////////////////////////*/
    error LockSlashingNotAuthorized();

    error LockSlashingStatusIsNotPending();

    error LockSlashingExpired();

    error LockSlashingResolutionNotReached();

    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Emitted when a new slash request is created.
     * @param service The address of the service that requested the slashing.
     * @param operator The address of the operator being slashed.
     * @param slashId The unique identifier for the slashing request.
     */
    event SlashingRequested(
        address indexed service, address indexed operator, bytes32 slashId, RequestInfo slashingInfo
    );

    /**
     * @dev Emitted when a slash request has been canceled.
     * @param service The address of the service that requested the slashing.
     * @param operator The address of the operator being slashed.
     * @param slashId The unique identifier for the slashing request.
     */
    event SlashingCanceled(address indexed service, address indexed operator, bytes32 slashId);

    /**
     * @dev Emitted when a slash request has been locked.
     * This event is emitted when the slashed collateral are moved from the operator's vaults to the router for further processing.
     */
    event SlashingLocked(address indexed service, address indexed operator, bytes32 slashId);

    /**
     * @title Slashing Status
     * @dev Enum representing the status of a slashing request.
     */
    enum Status {
        /// The slashing request is pending and has not been processed yet.
        Pending,
        /// The slashing request has been executed and funds are locked.
        Locked,
        /// The slashing request has been finalized.
        Finalized,
        /// The slashing request has been canceled.
        Canceled
    }

    /**
     * @title Slashing Request
     * @dev Payload for when a service request slashing of an operator.
     */
    struct Request {
        /// The operator address to slash.
        /// (service, operator) must have active registration at the timestamp.
        address operator;
        /// The percentage of tokens to slash in millis basis points (1/100,000th of a percent).
        /// Max millis bips to slash is set by the service slashing parameters {ISLAYRegistryV2.SlashParameter}
        /// at the timestamp and the operator must have opted in.
        uint24 mbips;
        /// The timestamp at which the slashing condition occurred.
        uint32 timestamp;
        /// Additional contextual information about the slashing request.
        Metadata metadata;
    }

    /**
     * @title Slashing Metadata
     * @dev Metadata is a struct that contains additional information about the slashing request.
     * Does not affect protocol logic, but can be used for logging or informational purposes.
     */
    struct Metadata {
        /// The reason for the slashing request.
        /// Must contain human-readable string.
        /// Max length of 250 characters, empty string is allowed but not recommended.
        string reason;
    }

    /**
     * @dev {RequestInfo} is a struct for internal state tracking.
     * Includes additional data besides the original slashing request payload.
     */
    struct RequestInfo {
        /// The service that initiated the slashing request.
        address service;
        /// The status of the slashing request.
        Status status;
        /// The timestamp when the request was submitted.
        uint32 requestTime;
        /// The timestamp when the request resolution window will end and becomes eligible for locking.
        /// This will be `requestTime` + `resolutionWindow`.
        uint32 requestResolution;
        /// The timestamp after which the request is no longer valid.
        /// This will be `requestTime` + `resolutionWindow` + `SLASHING_REQUEST_EXPIRY_WINDOW`
        uint32 requestExpiry;
        /// The core slashing request data including operator, bips, timestamp, and metadata.
        Request request;
    }

    /// @dev struct used internally to track locked assets in the router for further processing.
    struct LockedAssets {
        /// The amount of assets locked in the router.
        uint256 amount;
        /// The originating vault address from which the assets were slashed.
        address vault;
    }

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Get the current active slashing request for given service operator pair.
     * @param service Address of the service.
     * @param operator Address of the operator.
     * @return RequestInfo The current active slashing request information.
     */
    function getPendingSlashingRequest(address service, address operator) external view returns (RequestInfo memory);

    /**
     * @dev Get the locked assets for a given slash request.
     * @param slashId The unique identifier for the slash request.
     * @return LockedAssets[] An array of locked assets associated with the slashing request.
     */
    function getLockedAssets(bytes32 slashId) external view returns (LockedAssets[] memory);

    /**
     * @notice Initiates a slashing request against an active operator of the service (msg.sender).
     * This allows a registered service to request a slash of an operator's staked tokens
     * as a penalty for violations or non-compliance. The slashing request must meet several criteria:
     *
     * - Only callable by registered services.
     * - The service must be actively registered with the operator at the specified timestamp
     * - The slashing amount (in mbips) must not exceed the maxMbips set by the service
     * - The operator must have opted in to slashing at the specified timestamp
     * - The timestamp must be within the allowable slashing window (not too old or in the future)
     * - The service must not have another active slashing request against the same operator
     * - The reason provided in metadata must not exceed the maximum allowed length
     *
     * When successful, this creates a slashing request with an expiry time based on the
     * resolutionWindow parameter and returns a unique slashing request ID.
     * @param request The slashing request payload containing operator, mbips, timestamp, and metadata.
     * @return slashId The unique identifier for the slashing request.
     */
    function requestSlashing(Request calldata request) external returns (bytes32 slashId);

    /**
     * @dev Initiates the movement of slashed collateral from vaults to the router
     * which will later be finalized and handle according to the service slashing rules.
     * - Move all of operator's vaults slashed assets to the router for further processing.
     * - It can only be called after the resolution window has passed and before the expiry.
     * - Only callable by the service that initiated the slash request.
     *
     * @param slashId The unique identifier for the slash request.
     */
    function lockSlashing(bytes32 slashId) external;
}

/// @title Library for computing slashing request ID
library SlashingRequestId {
    /**
     * @dev generate a unique identifier for a slashing request.
     *
     * @param info The slashing request information.
     * @return bytes32 The computed hash of the slashing request.
     */
    function hash(ISLAYRouterSlashingV2.RequestInfo memory info) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                info.request.operator,
                info.request.mbips,
                info.request.timestamp,
                info.requestTime,
                info.requestResolution,
                info.requestExpiry,
                info.service
            )
        );
    }
}
