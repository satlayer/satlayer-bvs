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
     * @param request The information about the slashing request.
     * @param reason The reason for the slashing request, a human-readable string. Not stored on-chain.
     */
    event SlashingRequested(
        address indexed service, address indexed operator, bytes32 indexed slashId, Request request, string reason
    );

    /**
     * @dev Emitted when a slash request has been canceled.
     * @param service The address of the service that requested the slashing.
     * @param operator The address of the operator being slashed.
     * @param slashId The unique identifier for the slashing request.
     */
    event SlashingCanceled(address indexed service, address indexed operator, bytes32 indexed slashId);

    /**
     * @dev Emitted when a slash request has been locked.
     * This event is emitted when the slashed collateral are moved from the operator's vaults to the router for further processing.
     */
    event SlashingLocked(address indexed service, address indexed operator, bytes32 indexed slashId);

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
     * @dev slashing request payload is a struct that contains the necessary information
     * to initiate a slashing request. This struct is not stored on-chain, see {Request}.
     */
    struct Payload {
        /// The operator address to slash.
        /// The (service, operator) must have active registration at the timestamp.
        address operator;
        /// The percentage of tokens to slash in millis basis points (1/100,000th of a percent).
        /// Max millis bips to slash is set by the service slashing parameters {ISLAYRegistryV2.SlashParameter}
        /// at the timestamp and the operator must have opted in.
        uint24 mbips;
        /// The real timestamp at which the slashing condition occurred.
        /// This timestamp does not have to be tied to the block timestamp.
        uint32 timestamp;
        /// The reason for the slashing request, must be a human-readable string max length of 250 characters.
        //  The reason is for informational purposes, not stored on-chain, emitted in events.
        string reason;
    }

    /**
     * @dev Slashing request is a struct for internal state tracking.
     * Includes additional data besides the original slashing request payload.
     *
     * Slot 0:
     * - status: uint8 (8 bits)
     * - service: address (160 bits)
     * - mbips: uint24 (24 bits)
     * - timestamp: uint32 (32 bits)
     * - requestTime: uint32 (32 bits)
     *
     * Slot 1:
     * - operator: address (160 bits)
     * - requestResolution: uint32 (32 bits)
     * - requestExpiry: uint32 (32 bits)
     */
    struct Request {
        /// The current status of the slashing request.
        Status status;
        /// The service that initiated the slashing request.
        address service;
        /// The percentage of tokens to slash in millis basis points (1/100,000th of a percent).
        /// Max millis bips to slash is set by the service slashing parameters {ISLAYRegistryV2.SlashParameter}
        /// at the timestamp and the operator must have opted in.
        uint24 mbips;
        /// The real timestamp at which the slashing condition occurred.
        /// This timestamp does not have to be tied to the block timestamp.
        uint32 timestamp;
        /// The timestamp when the request was submitted.
        /// This is block timestamp when the slashing request was made.
        uint32 requestTime;
        /// The operator to slash.
        address operator;
        /// The timestamp when the request resolution window will end and becomes eligible for locking.
        /// This will be `requestTime` + `resolutionWindow`.
        uint32 requestResolution;
        /// The timestamp after which the request is no longer valid.
        /// This will be `requestTime` + `resolutionWindow` + `SLASHING_REQUEST_EXPIRY_WINDOW`
        uint32 requestExpiry;
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
    function getPendingSlashingRequest(address service, address operator) external view returns (Request memory);

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
     *
     * @param payload The slashing request payload containing the operator, mbips, timestamp, and reason.
     * @return slashId The unique identifier for the slashing request.
     */
    function requestSlashing(Payload calldata payload) external returns (bytes32 slashId);

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

/// @title Slashing Request ID Library
/// @dev Single purpose library to compute a unique identifier for slashing requests.
library SlashingRequestId {
    /**
     * @dev generate a unique identifier for a slashing request.
     *
     * @param request The slashing request information.
     * @return bytes32 The computed hash of the slashing request.
     */
    function hash(ISLAYRouterSlashingV2.Request memory request) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                request.service,
                request.mbips,
                request.timestamp,
                request.requestTime,
                request.operator,
                request.requestResolution,
                request.requestExpiry
            )
        );
    }
}
