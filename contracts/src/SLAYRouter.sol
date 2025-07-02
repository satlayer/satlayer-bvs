// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

import {SLAYRegistry, ServiceOperator, SlashParameter} from "./SLAYRegistry.sol";
import {ISLAYRouter} from "./interface/ISLAYRouter.sol";
import {ISLAYRegistry} from "./interface/ISLAYRegistry.sol";

/**
 * @title SLAYRouter
 * @dev The central point for managing interactions with SLAYVaults.
 * This contract is designed to work with the SLAYRegistry for managing vaults and their states.
 *
 * @custom:oz-upgrades-from src/InitialImpl.sol:InitialImpl
 */
contract SLAYRouter is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable, ISLAYRouter {
    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRegistry public immutable registry;

    /**
     * @notice 7 days
     */
    uint32 public constant slashingRequestExpiryWindow = 7 * 24 * 60 * 60;

    mapping(address => bool) public whitelisted;

    mapping(bytes32 serviceOperatorKey => bytes32 slashId) public slashingRequestIds;

    mapping(bytes32 slashId => Slashing.RequestInfo) public slashingRequests;

    modifier onlyValidSlashRequest(Slashing.RequestPayload memory request) {
        ServiceOperator.Relationship memory rs =
            registry.getRelationshipAt(_msgSender(), request.operator, request.timestamp);
        require(rs.slashOptedIn == true, "Operator has not opted in to the slash at specified timestamp.");
        require(
            rs.status == ISLAYRegistry.RegistrationStatus.Active,
            "Service and Operator must be active at the specified timestamp"
        );
        SlashParameter.Object memory param = registry.getSlashParameterAt(_msgSender(), request.timestamp);
        require(request.millieBips <= param.maxMilliBips, "Slash requested amount is more than the service has allowed");

        uint32 withdrawalDelay = registry.getWithdrawalDelay(request.operator);

        require(
            request.timestamp > (block.timestamp - withdrawalDelay),
            "Slash timestamp must be within the allowable slash period"
        );

        require(request.timestamp <= block.timestamp, "Cannot request slash with timestamp greater than present");
        _;
    }

    /**
     * @dev Set the immutable SLAYRegistry proxy address for the implementation.
     * Cyclic params in constructor are possible as an InitialImpl (empty implementation) is used for an initial deployment,
     * after which all the contracts are upgraded to their respective implementations with immutable proxy addresses.
     *
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(SLAYRegistry registry_) {
        registry = registry_;
        _disableInitializers();
    }

    /**
     * @dev Authorizes an upgrade to a new implementation.
     * This function is required by UUPS and restricts upgradeability to the contract owner.
     * @param newImplementation The address of the new contract implementation.
     */
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    /**
     * @dev Pauses the contract, all SLAYVaults will also be paused.
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @dev Unpauses the contract, all SLAYVaults will also be unpaused.
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    /// @inheritdoc ISLAYRouter
    function setVaultWhitelist(address vault_, bool isWhitelisted) external onlyOwner {
        whitelisted[vault_] = isWhitelisted;
        emit VaultWhitelisted(vault_, isWhitelisted);
    }

    function requestSlashing(Slashing.RequestPayload memory payload) public onlyValidSlashRequest(payload) {
        address service = _msgSender();
        Slashing.RequestInfo memory pendingSlashingRequest = getPendingSlashingRequest(service, payload.operator);

        if (Slashing.isRequestInfoExist(pendingSlashingRequest) == true) {
            if (
                pendingSlashingRequest.status == Slashing.RequestStatus.Pending
                    && pendingSlashingRequest.requestExpiry > block.timestamp
            ) {
                revert("Pending Slashing Request lifecycle not completed");
            }
        }

        uint32 requestResolution =
            uint32(block.timestamp) + registry.getSlashParameterAt(service, payload.timestamp).resolutionWindow;
        uint32 requestExpiry = requestResolution + slashingRequestExpiryWindow;

        Slashing.RequestInfo memory newSlashingRequestInfo = Slashing.RequestInfo({
            request: payload,
            requestTime: uint32(block.timestamp),
            requestResolution: requestResolution,
            requestExpiry: requestExpiry,
            status: Slashing.RequestStatus.Pending,
            service: service
        });

        _updateSlashingRequest(service, payload.operator, newSlashingRequestInfo);
    }

    function getPendingSlashingRequest(address service, address operator)
        public
        view
        returns (Slashing.RequestInfo memory)
    {
        bytes32 key = ServiceOperator._getKey(service, operator);
        bytes32 slashId = slashingRequestIds[key];
        return slashingRequests[slashId];
    }

    function _updateSlashingRequest(address service, address operator, Slashing.RequestInfo memory info) internal {
        bytes32 key = ServiceOperator._getKey(service, operator);
        bytes32 slashId = Slashing.calculateSlashingRequestId(info);
        slashingRequestIds[key] = slashId;
        slashingRequests[slashId] = info;
    }
}

library Slashing {
    enum RequestStatus {
        Pending,
        Locked,
        Canceled,
        Finalized
    }

    struct RequestPayload {
        address operator;
        uint32 millieBips;
        uint32 timestamp;
        MetaData metaData;
    }

    struct RequestInfo {
        RequestPayload request;
        uint32 requestTime;
        uint32 requestResolution;
        uint32 requestExpiry;
        RequestStatus status;
        address service;
    }

    struct MetaData {
        string reason;
    }

    function isRequestInfoExist(RequestInfo memory info) public pure returns (bool) {
        if (
            info.service == address(0) && info.request.operator == address(0) && info.requestTime == 0
                && info.requestResolution == 0 && info.requestExpiry == 0
        ) {
            return true;
        }
        return false;
    }

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
                uint8(info.status),
                info.service
            )
        );
    }
}
