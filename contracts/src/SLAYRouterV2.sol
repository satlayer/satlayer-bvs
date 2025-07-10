// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {ISLAYRegistryV2} from "./interface/ISLAYRegistryV2.sol";
import {ISLAYRouterV2} from "./interface/ISLAYRouterV2.sol";
import {ISLAYVaultV2} from "./interface/ISLAYVaultV2.sol";
import {ISLAYSlashingV2, SlashingRequest} from "./interface/ISLAYSlashingV2.sol";

/**
 * @title Vaults Router Contract
 * @dev The central point for managing interactions with SLAYVaults.
 * This contract is designed to work with the SLAYRegistryV2 for managing vaults and their states.
 *
 * @custom:oz-upgrades-from src/InitialImpl.sol:InitialImpl
 */
contract SLAYRouterV2 is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable,
    ISLAYRouterV2,
    ISLAYSlashingV2
{
    using EnumerableSet for EnumerableSet.AddressSet;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    ISLAYRegistryV2 public immutable registry;

    /// @dev Whitelisted flag for each vault.
    mapping(address => bool) internal _whitelisted;

    /**
     * @notice 7 days
     */
    uint32 public constant slashingRequestExpiryWindow = 7 days;

    /// @dev The max number of vaults allowed per operator.
    uint8 private _maxVaultsPerOperator;

    /// @dev Stores the EnumerableSet of vault address for each operator.
    mapping(address operator => EnumerableSet.AddressSet) private _operatorVaults;

    /// @dev Stores the id for most recent slashing request for a given service operator pair.
    mapping(address service => mapping(address operator => bytes32)) private _pendingSlashingRequestIds;

    /// @dev Stores the slashing requests by its id.
    mapping(bytes32 slashId => ISLAYSlashingV2.RequestInfo) private _slashingRequests;

    modifier onlyService(address account) {
        if (!registry.isService(account)) {
            revert ISLAYRegistryV2.ServiceNotFound(account);
        }
        _;
    }

    /**
     * @dev Set the immutable SLAYRegistryV2 proxy address for the implementation.
     * Cyclic params in constructor are possible as an InitialImpl (empty implementation) is used for an initial deployment,
     * after which all the contracts are upgraded to their respective implementations with immutable proxy addresses.
     *
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(ISLAYRegistryV2 registry_) {
        registry = registry_;
        _disableInitializers();
    }

    /**
     * @dev Initializes SLAYRouterV2 contract.
     * Set the default max vaults per operator to 10.
     */
    function initialize() public reinitializer(2) {
        _maxVaultsPerOperator = 10;
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

    /// @inheritdoc ISLAYRouterV2
    function getMaxVaultsPerOperator() external view returns (uint8) {
        return _maxVaultsPerOperator;
    }

    /// @inheritdoc ISLAYRouterV2
    function setMaxVaultsPerOperator(uint8 count) external onlyOwner {
        require(count > _maxVaultsPerOperator, "Must be greater than current");
        _maxVaultsPerOperator = count;
    }

    /// @inheritdoc ISLAYRouterV2
    function setVaultWhitelist(address vault_, bool isWhitelisted) external onlyOwner {
        require(_whitelisted[vault_] != isWhitelisted, "Vault already in desired state");

        address operator = ISLAYVaultV2(vault_).delegated();
        EnumerableSet.AddressSet storage vaults = _operatorVaults[operator];

        if (isWhitelisted) {
            require(vaults.length() < _maxVaultsPerOperator, "Exceeds max vaults per operator");

            vaults.add(vault_);
        } else {
            vaults.remove(vault_);
        }

        _whitelisted[vault_] = isWhitelisted;
        emit VaultWhitelisted(operator, vault_, isWhitelisted);
    }

    /// @inheritdoc ISLAYRouterV2
    function isVaultWhitelisted(address vault_) external view returns (bool) {
        return _whitelisted[vault_];
    }

    /// @inheritdoc ISLAYSlashingV2
    function requestSlashing(ISLAYSlashingV2.Request calldata payload)
        external
        onlyService(_msgSender())
        whenNotPaused
    {
        _checkSlashRequest(payload);
        address service = _msgSender();
        ISLAYSlashingV2.RequestInfo memory pendingSlashingRequest = getPendingSlashingRequest(service, payload.operator);

        if (SlashingRequest.isRequestInfoExist(pendingSlashingRequest) == true) {
            if (pendingSlashingRequest.status == ISLAYSlashingV2.Status.Pending) {
                if (pendingSlashingRequest.requestExpiry > uint32(block.timestamp)) {
                    // previous slashing request is pending within expiry date or has locked
                    revert("Previous slashing request lifecycle not completed");
                } else {
                    // previous slashing request is pending but expired
                    // eligible for new slashing request to take place
                    // by canceling the previous slashing request.
                    _cancelSlashingRequest(service, payload.operator, pendingSlashingRequest);
                }
            }

            if (pendingSlashingRequest.status == ISLAYSlashingV2.Status.Locked) {
                revert("Previous slashing request lifecycle not completed");
            }
        }

        uint32 requestResolution = uint32(block.timestamp)
            + registry.getSlashParameterAt(service, payload.operator, payload.timestamp).resolutionWindow;
        uint32 requestExpiry = requestResolution + slashingRequestExpiryWindow;

        ISLAYSlashingV2.RequestInfo memory newSlashingRequestInfo = ISLAYSlashingV2.RequestInfo({
            request: payload,
            requestTime: uint32(block.timestamp),
            requestResolution: requestResolution,
            requestExpiry: requestExpiry,
            status: ISLAYSlashingV2.Status.Pending,
            service: service
        });

        _createNewSlashingRequest(service, payload.operator, newSlashingRequestInfo);
    }

    /**
     * Gets current active slashing request for given service operator pair.
     * @param service Address of the service.
     * @param operator Address of the operator.
     */
    function getPendingSlashingRequest(address service, address operator)
        public
        view
        returns (ISLAYSlashingV2.RequestInfo memory)
    {
        bytes32 slashId = _pendingSlashingRequestIds[service][operator];
        return _slashingRequests[slashId];
    }

    /**
     * Create a new slashing request
     * @param service Address of the service.
     * @param operator Address of the operator.
     * @param info {ISLAYRouter.ISLAYSlashingV2.RequestInfo}
     */
    function _createNewSlashingRequest(address service, address operator, ISLAYSlashingV2.RequestInfo memory info)
        internal
        returns (bytes32)
    {
        bytes32 slashId = SlashingRequest.calculateSlashingRequestId(info);
        _pendingSlashingRequestIds[service][operator] = slashId;
        _slashingRequests[slashId] = info;
        emit ISLAYSlashingV2.SlashingRequested(service, operator, slashId, info);
        return slashId;
    }

    /**
     * Cancel Slashing request
     * @param service Address of the service.
     * @param operator Address of the operator.
     * @param pendingSlashingRequest {ISLAYRouter.ISLAYSlashingV2.RequestInfo}
     */
    function _cancelSlashingRequest(
        address service,
        address operator,
        ISLAYSlashingV2.RequestInfo memory pendingSlashingRequest
    ) internal {
        pendingSlashingRequest.status = ISLAYSlashingV2.Status.Canceled;
        bytes32 slashId = SlashingRequest.calculateSlashingRequestId(pendingSlashingRequest);
        delete _pendingSlashingRequestIds[service][operator];

        _slashingRequests[slashId] = pendingSlashingRequest;
        emit ISLAYSlashingV2.SlashingCanceled(service, operator, slashId, pendingSlashingRequest);
    }

    function _checkSlashRequest(ISLAYSlashingV2.Request memory request) internal {
        ISLAYRegistryV2.SlashParameter memory slashBounds =
            registry.getSlashParameterAt(_msgSender(), request.operator, request.timestamp);

        require(request.mbips <= slashBounds.maxMbips, "Slash requested amount is more than the service has allowed");
        require(request.mbips > 0, "Requested slashing amount in milli bips must be greater than zero");

        uint32 withdrawalDelay = registry.getWithdrawalDelay(request.operator);

        require(
            request.timestamp > (block.timestamp - withdrawalDelay),
            "Slash timestamp must be within the allowable slash period"
        );

        require(request.timestamp <= block.timestamp, "Cannot request slash with timestamp greater than present");
    }
}
