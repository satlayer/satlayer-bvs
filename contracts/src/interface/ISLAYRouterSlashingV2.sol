// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

/**
 * @title SLAY Router Slashing V2 Interface
 * @dev Interface for the slashing functionality in the SatLayer protocol.
 * This interface defines the structure and functions for slashing requests, which are penalties
 * applied to operators for violations or non-compliance.
 * It is designed to be implemented on the SLAYRouter, but separated to allow for clear
 * separation of slashing-related concerns from other router functionality.
 */
interface ISLAYRouterSlashingV2 {
    /*//////////////////////////////////////////////////////////////
                                ERRORS
    //////////////////////////////////////////////////////////////*/

    /// @dev Error thrown when an unauthorized address attempts to perform a restricted operation.
    error Unauthorized();

    /// @dev Error thrown when an operation is attempted on a slashing request with an invalid status.
    error InvalidStatus();

    /// @dev Error thrown when an operation is attempted on a slashing request that has expired.
    error SlashingRequestExpired();

    /// @dev Error thrown when an operation references a slashing request that does not exist.
    error SlashingRequestNotFound();

    /// @dev Error thrown when attempting to lock a slashing request before its resolution window has passed.
    error SlashingResolutionNotReached();

    /// @dev Error thrown when guardrail attempts to approve/reject a slashing request more than once.
    error GuardrailAlreadyApproved();

    /// @dev Error thrown when attempting to finalize a slashing request that has not been approved by the guardrail.
    error GuardrailHaveNotApproved();

    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Emitted when a new slashing request is created.
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
     * @dev Emitted when a slashing request has been canceled.
     * This occurs when a service explicitly cancels a pending request or when a new request
     * replaces an expired one.
     * @param service The address of the service that requested the slashing.
     * @param operator The address of the operator that was targeted by the canceled request.
     * @param slashId The unique identifier for the canceled slashing request.
     */
    event SlashingCanceled(address indexed service, address indexed operator, bytes32 indexed slashId);

    /**
     * @dev Emitted when a slashing request has been locked.
     * This event is emitted when the slashed collateral is moved from the operator's vaults
     * to the router for further processing. Locking occurs after the resolution window has passed
     * and before the request expires.
     * @param service The address of the service that requested the slashing.
     * @param operator The address of the operator whose collateral is being locked.
     * @param slashId The unique identifier for the locked slashing request.
     */
    event SlashingLocked(address indexed service, address indexed operator, bytes32 indexed slashId);

    /**
     * @dev Emitted when a slashing request has been finalized.
     * This event is emitted when the slashed collateral is moved from the router to the destination address.
     * The destination address is agreed upon by the service and the operator in the slashing parameters.
     * Finalization can only occur after locking and guardrail approval.
     * @param service The address of the service that requested the slashing.
     * @param operator The address of the operator whose collateral was slashed.
     * @param slashId The unique identifier for the finalized slashing request.
     * @param destination The address to which the slashed collateral was sent.
     */
    event SlashingFinalized(
        address indexed service, address indexed operator, bytes32 indexed slashId, address destination
    );

    /**
     * @dev Emitted when a slashing request has been approved or rejected by the guardrail.
     * The guardrail is a security mechanism that provides additional approval for slashing operations.
     * @param slashId The unique identifier for the slashing request that received a decision.
     * @param approval True if the guardrail approved the slashing request, false if rejected.
     */
    event GuardrailApproval(bytes32 indexed slashId, bool approval);

    /**
     * @title Slashing Status
     * @dev Enum representing the possible states of a slashing request throughout its lifecycle.
     * The status transitions follow this flow:
     * - Pending -> Locked -> Finalized
     * - Pending -> Canceled
     */
    enum Status {
        /// The slashing request is pending and has not been processed yet.
        /// This is the initial state when a request is created.
        Pending,
        /// The slashing request has been executed and funds are locked in the router.
        /// This state occurs after the resolution window has passed and before finalization.
        Locked,
        /// The slashing request has been finalized and funds have been transferred to the destination.
        /// This is the terminal state for a successful slashing request.
        Finalized,
        /// The slashing request has been canceled and will not be processed further.
        /// This can happen due to explicit cancellation or replacement by a new request.
        Canceled
    }

    /**
     * @title Slashing Request Payload
     * @dev Contains the necessary information to initiate a slashing request.
     * This struct is used as an input parameter and is not stored on-chain directly.
     * See {Request} for the on-chain storage structure.
     */
    struct Payload {
        /// @dev The operator address to slash.
        /// The (service, operator) pair must have an active registration at the specified timestamp.
        address operator;
        /// @dev The percentage of tokens to slash in millis basis points (1/100,000th of a percent).
        /// For example, 10,000 mbips = 0.1%, 1,000,000 mbips = 10%.
        /// Max millis bips to slash is set by the service slashing parameters {ISLAYRegistryV2.SlashParameter}
        /// at the timestamp, and the operator must have opted in to slashing.
        uint24 mbips;
        /// @dev The real timestamp at which the slashing condition occurred.
        /// This timestamp does not have to be tied to the block timestamp but must be within
        /// the allowable slashing window (not too old or in the future).
        uint32 timestamp;
        /// @dev The reason for the slashing request, must be a human-readable string with max length of 250 characters.
        /// The reason is for informational purposes only, not stored on-chain, and is emitted in events.
        string reason;
    }

    /**
     * @title Slashing Request
     * @dev Represents a slashing request stored on-chain for internal state tracking.
     * Includes all data from the original payload plus additional tracking information.
     * The struct is optimized for gas efficiency by packing related fields together.
     *
     * Memory layout:
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
        /// @dev The current status of the slashing request (Pending, Locked, Finalized, or Canceled).
        Status status;
        /// @dev The address of the service that initiated the slashing request.
        /// Only this service can lock, finalize, or cancel the request.
        address service;
        /// @dev The percentage of tokens to slash in millis basis points (1/100,000th of a percent).
        uint24 mbips;
        /// @dev The real timestamp at which the slashing condition occurred.
        uint32 timestamp;
        /// @dev The block timestamp when the slashing request was submitted.
        /// Used as the basis for calculating resolution and expiry windows.
        uint32 requestTime;
        /// @dev The address of the operator being slashed.
        address operator;
        /// @dev The timestamp when the request resolution window ends and the request becomes eligible for locking.
        /// Calculated as `requestTime` + `resolutionWindow` from the slash parameters.
        uint32 requestResolution;
        /// @dev The timestamp after which the request is no longer valid and cannot be locked.
        /// Calculated as `requestResolution` + `SLASHING_REQUEST_EXPIRY_WINDOW`.
        uint32 requestExpiry;
    }

    /**
     * @title Locked Assets
     * @dev Struct used to track assets that have been locked in the router during the slashing process.
     * These assets are temporarily held by the router before being transferred to the final destination
     * during the finalization step.
     */
    struct LockedAssets {
        /// @dev The amount of assets locked in the router, denominated in the vault's asset token.
        uint256 amount;
        /// @dev The address of the vault from which the assets were originally locked.
        /// Used to identify the source of the funds and the associated asset token.
        address vault;
    }

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Returns the most recent slashing request initiated by the specified service against the specified operator.
     * If no active request exists, the returned request will have default values.
     * @param service Address of the service that initiated the slashing request.
     * @param operator Address of the operator targeted by the slashing request.
     * @return A Request struct containing the details of the current active slashing request.
     */
    function getPendingSlashingRequest(address service, address operator) external view returns (Request memory);

    /**
     * @dev Returns the complete details of a slashing request identified by the provided slashId.
     * If no request exists with the given slashId, the returned request will have default values.
     * @param slashId The unique identifier for the slashing request to retrieve.
     * @return A Request struct containing the details of the specified slashing request.
     */
    function getSlashingRequest(bytes32 slashId) external view returns (Request memory);

    /**
     * @dev Returns an array of LockedAssets structs representing the assets that have been locked
     * as part of the slashing process. This information is only relevant for requests in the Locked status.
     * @param slashId The unique identifier for the slashing request.
     * @return An array of LockedAssets structs containing information about the locked assets.
     */
    function getLockedAssets(bytes32 slashId) external view returns (LockedAssets[] memory);

    /**
     * @dev Allows a registered service to request a slash of an operator's staked tokens
     * as a penalty for violations or non-compliance. The slashing request must meet several criteria:
     *
     * - Only callable by registered services.
     * - The service must be actively registered with the operator at the specified timestamp.
     * - The slashing amount (in mbips) must not exceed the maxMbips set by the service.
     * - The operator must have opted in to slashing at the specified timestamp.
     * - The timestamp must be within the allowable slashing window (not too old or in the future).
     * - The service must not have another active slashing request against the same operator.
     * - The reason provided must not exceed the maximum allowed length (250 characters).
     *
     * When successful, this creates a slashing request with an expiry time based on the
     * resolutionWindow parameter and returns a unique slashing request ID.
     *
     * @param payload The slashing request payload containing the operator, mbips, timestamp, and reason.
     * @return slashId The unique identifier for the newly created slashing request.
     */
    function requestSlashing(Payload calldata payload) external returns (bytes32 slashId);

    /**
     * @dev Initiates the movement of slashed collateral from the operator's vaults to the router
     * for temporary holding before finalization. This function:
     *
     * - Moves the calculated portion of the operator's assets from all their vaults to the router.
     * - Can only be called after the resolution window has passed and before the request expires.
     * - Can only be called by the service that initiated the slashing request.
     * - Changes the request status from Pending to Locked.
     *
     * The amount locked from each vault is calculated based on the mbips value in the request.
     *
     * @param slashId The unique identifier for the slashing request to lock.
     */
    function lockSlashing(bytes32 slashId) external;

    /**
     * @dev Completes the slashing process by moving the locked assets from the router to the
     * destination address specified in the slashing parameters. This function:
     *
     * - Can only be called by the service that initiated the slashing request.
     * - Can only be executed if the request is in the Locked status.
     * - Requires prior approval from the guardrail.
     * - Changes the request status from Locked to Finalized.
     *
     * After finalization, the slashing process is complete and cannot be reversed.
     *
     * @param slashId The unique identifier for the slashing request to finalize.
     */
    function finalizeSlashing(bytes32 slashId) external;

    /**
     * @dev Allows the designated guardrail address to approve or reject a slashing request
     * before it can be finalized. This function:
     *
     * - Can only be called by the designated guardrail address.
     * - Can only be called once per slashing request.
     * - Does not check the status of the request (this will be checked during finalization).
     * - Records the approval decision for later verification during finalization.
     *
     * The guardrail serves as an additional security mechanism to prevent unauthorized slashing.
     *
     * @param slashId The unique identifier for the slashing request to approve or reject.
     * @param approve True to approve the slashing request, false to reject it.
     */
    function guardrailApprove(bytes32 slashId, bool approve) external;

    /**
     * @dev Allows a service to cancel its own pending slashing request. This function:
     *
     * - Can only be called by the service that initiated the slashing request.
     * - Can only be called if the request is in the Pending status.
     * - Changes the request status from Pending to Canceled.
     * - Removes the request from the pending requests mapping.
     *
     * Once canceled, a slashing request cannot be reactivated, but a new request can be created.
     *
     * @param slashId The unique identifier for the slashing request to cancel.
     */
    function cancelSlashing(bytes32 slashId) external;
}

/// @title Slashing Request ID Library
library SlashingRequestId {
    /**
     * @dev Computes a keccak256 hash of the essential fields of a slashing request.
     * The hash includes all fields that uniquely identify a request, excluding the status field
     * which can change during the request lifecycle.
     *
     * The fields included in the hash are:
     * - service: The address of the service that initiated the request
     * - mbips: The percentage of tokens to slash presented in millis basis points
     * - timestamp: The timestamp when the slashing condition occurred
     * - requestTime: The timestamp when the request was submitted
     * - operator: The address of the operator being slashed
     * - requestResolution: The timestamp when the resolution window ends
     * - requestExpiry: The timestamp when the request expires
     *
     * @param request The slashing request struct containing all the necessary information.
     * @return A bytes32 hash that uniquely identifies the slashing request.
     */
    function hash(ISLAYRouterSlashingV2.Request memory request) internal pure returns (bytes32) {
        /// We don't use inline assembly for keccak256 for this hashing function,
        /// due to the minimal gas savings and it doesn't fit into scratch space.
        /// It's better to maintain readability and security of the code.
        /// forge-lint: disable-start(asm-keccak256)
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
        /// forge-lint: disable-end(asm-keccak256)
    }
}
