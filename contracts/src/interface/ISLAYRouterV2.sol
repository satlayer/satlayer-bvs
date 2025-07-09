// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {SLAYRegistryV2} from "../SLAYRegistryV2.sol";

/**
 * @title Vaults Router Interface
 * @dev Interface for the SLAYRouterV2 contract.
 */
interface ISLAYRouterV2 {
    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Emitted when the vault whitelist status is updated.
     */
    event VaultWhitelisted(address indexed operator, address vault, bool whitelisted);

    /**
     * @dev Emitted when new slash request is accepted.
     */
    event NewSlashingRequest(
        address indexed service, address indexed operator, bytes32 indexed slashId, Slashing.RequestInfo slashingInfo
    );

    /**
     * @dev Emitted when new slash request is being canceled.
     */
    event CancelSlashingRequest(
        address indexed service, address indexed operator, bytes32 indexed slashId, Slashing.RequestInfo slashingInfo
    );

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @return The max number of vaults allowed per operator.
     */
    function getMaxVaultsPerOperator() external view returns (uint8);

    /**
     * @dev Update the max number of vaults allowed per operator.
     * The new value must be greater than the previous value.
     * @param count The new maximum number of vaults per operator.
     */
    function setMaxVaultsPerOperator(uint8 count) external;

    /**
     * @dev Set the individual whitelist status for a SLAYVault.
     * This allows CA owner to control which vaults can be interacted with through the router.
     * For non-granular state/modifier, use {SLAYRouterV2-pause} to pause all vaults.
     * When a vault is whitelisted, it can be interacted with through the router.
     *
     * @param vault_ address of the vault to set the whitelist status for.
     * This should be a SLAYVault contract address but isn't "checked" to allow for flexible un-whitelisting of vaults.
     * @param isWhitelisted The whitelist status to set.
     */
    function setVaultWhitelist(address vault_, bool isWhitelisted) external;

    /**
     * Request slashing by the service to an misbehaving operator.
     * Slashing request for a given operator by the same service can only take place one after another.
     * @param payload {Slashing.RequestPayload}
     */
    function requestSlashing(Slashing.RequestPayload memory payload) external;

    /**
     * @dev Check if a vault is whitelisted.
     * @param vault_ The address of the vault to check.
     * @return True if the vault is whitelisted, false otherwise.
     */
    function isVaultWhitelisted(address vault_) external view returns (bool);
}

library Slashing {
    enum RequestStatus {
        /**
         * Earliest stage of a slashing request's lifecycle.
         */
        Pending,
        /**
         * Locked stage is where the slashed collateral are escrowed to SatLayer.
         */
        Locked,
        /**
         * Finalized stage is when slashed callateral are moved to destination address.
         */
        Finalized,
        /**
         * Slashing request is canceled when operator as refute adhering to BVS's protocol.
         * Slashing request is also canceled when service has failed to take action beyond expiry.
         */
        Canceled
    }

    /**
     * {RequestPayload} is a payload for when service request slashing.
     */
    struct RequestPayload {
        /**
         * Accused Operator's address.
         */
        address operator;
        /**
         * Collateral amount to be slashed.
         * Unit is in milli bips.
         * Cannot be more than service's slashing parameter bounds.
         */
        uint32 millieBips;
        /**
         * Timestamp at which alleged misbhaviour occurs.
         */
        uint32 timestamp;
        /**
         * Metadata associated to particular slashing request.
         */
        MetaData metaData;
    }

    struct MetaData {
        string reason;
    }

    /**
     * {RequestInfo} is a struct for internal state tracking.
     * Includes additional data besides the original slashing request payload.
     */
    struct RequestInfo {
        RequestPayload request;
        uint32 requestTime;
        uint32 requestResolution;
        uint32 requestExpiry;
        RequestStatus status;
        address service;
    }

    /**
     * Checks whether a given {RequestInfo} struct is solidity null defaults.
     * @param info  {RequestInfo}
     * Returns a boolean.
     */
    function isRequestInfoExist(RequestInfo memory info) public pure returns (bool) {
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
    function calculateSlashingRequestId(RequestInfo memory info) public pure returns (bytes32) {
        return keccak256(
            abi.encodePacked(
                info.request.operator,
                info.request.millieBips,
                info.request.timestamp,
                info.request.metaData.reason,
                info.requestTime,
                info.requestResolution,
                info.requestExpiry,
                info.service
            )
        );
    }
}
