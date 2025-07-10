// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

/**
 * @title SLAY Slashing V2 Interface
 * @dev Interface for the SLAYSlashingV2 contract.
 * This interface defines the structure and functions for slashing requests in the SatLayer protocol.
 * To be implemented on the SLAYRouter, but separated to allow for separate of slashing related concerns.
 */
interface ISLAYSlashingV2 {
    /**
     * @dev Emitted when a new slash request is created.
     */
    event SlashingRequested(
        address indexed service, address indexed operator, bytes32 indexed slashId, RequestInfo slashingInfo
    );

    /**
     * @dev Emitted when a slash request has been canceled.
     */
    event SlashingCanceled(
        address indexed service, address indexed operator, bytes32 indexed slashId, RequestInfo slashingInfo
    );

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
     * Request is a payload for when service request slashing.
     */
    struct Request {
        /**
         * Accused Operator's address.
         */
        address operator;
        /**
         * Collateral amount to be slashed.
         * Unit is in milli bips.
         * Cannot be more than service's slashing parameter bounds.
         */
        uint32 mbips;
        /**
         * Timestamp at which alleged misbehaviour occurs.
         */
        uint32 timestamp;
        /**
         * Metadata associated to particular slashing request.
         */
        Metadata metadata;
    }

    struct Metadata {
        string reason;
    }

    /**
     * {RequestInfo} is a struct for internal state tracking.
     * Includes additional data besides the original slashing request payload.
     */
    struct RequestInfo {
        Request request;
        uint32 requestTime;
        uint32 requestResolution;
        uint32 requestExpiry;
        Status status;
        address service;
    }

    /**
     * @dev Request slashing from a service to an misbehaving operator.
     * Slashing request for a given operator by the same service can only take place one after another.
     * @param payload {Slashing.RequestPayload}
     */
    function requestSlashing(Request calldata payload) external;
}

library SlashingRequest {
    /**
     * Checks whether a given {RequestInfo} struct is solidity null defaults.
     * @param info  {RequestInfo}
     * Returns a boolean.
     */
    function isRequestInfoExist(ISLAYSlashingV2.RequestInfo memory info) external pure returns (bool) {
        if (
            info.service == address(0) && info.request.operator == address(0) && info.requestTime == 0
                && info.requestResolution == 0 && info.requestExpiry == 0
        ) {
            return false;
        }
        return true;
    }

    /**
     * Hash the slashing request data to be used as an identifier within the protocol.
     * The function dismisses {RequestStatus} from hash function.
     * @param info  {RequestInfo}
     */
    function calculateSlashingRequestId(ISLAYSlashingV2.RequestInfo memory info) external pure returns (bytes32) {
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
