// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/utils/math/Math.sol";

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {ISLAYRegistryV2} from "./interface/ISLAYRegistryV2.sol";
import {ISLAYRouterV2} from "./interface/ISLAYRouterV2.sol";
import {ISLAYVaultV2} from "./interface/ISLAYVaultV2.sol";
import {ISLAYRouterSlashingV2, SlashingRequestId} from "./interface/ISLAYRouterSlashingV2.sol";

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
    ISLAYRouterSlashingV2
{
    using EnumerableSet for EnumerableSet.AddressSet;

    /**
     * @notice 7 days, the expiry window for slashing requests.
     */
    uint32 public constant SLASHING_REQUEST_EXPIRY_WINDOW = 7 days;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    ISLAYRegistryV2 public immutable registry;

    /// @dev Whitelisted flag for each vault.
    mapping(address => bool) internal _whitelisted;

    /// @dev The max number of vaults allowed per operator.
    uint8 private _maxVaultsPerOperator;

    /// @dev Stores the EnumerableSet of vault address for each operator.
    mapping(address operator => EnumerableSet.AddressSet) private _operatorVaults;

    /// @dev Stores the id for most recent slashing request for a given service operator pair.
    mapping(address service => mapping(address operator => bytes32)) private _pendingSlashingRequestIds;

    /// @dev Stores the slashing requests by its id.
    mapping(bytes32 slashId => ISLAYRouterSlashingV2.Request) private _slashingRequests;

    /// @dev Stores the locked assets for each slashing request.
    mapping(bytes32 slashId => ISLAYRouterSlashingV2.LockedAssets[]) private _lockedAssets;

    /// @dev Modifier to check if the caller is a registered service.
    /// Information is sourced from checking the SLAYRegistryV2 contract.
    /// @param account The address to check if it is a service.
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
    function getMaxVaultsPerOperator() external view override returns (uint8) {
        return _maxVaultsPerOperator;
    }

    /// @inheritdoc ISLAYRouterV2
    function setMaxVaultsPerOperator(uint8 count) external override onlyOwner {
        require(count > _maxVaultsPerOperator, "Must be greater than current");
        _maxVaultsPerOperator = count;
    }

    /// @inheritdoc ISLAYRouterV2
    function setVaultWhitelist(address vault_, bool isWhitelisted) external override onlyOwner {
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
    function isVaultWhitelisted(address vault_) external view override returns (bool) {
        return _whitelisted[vault_];
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function getPendingSlashingRequest(address service, address operator)
        external
        view
        override
        returns (ISLAYRouterSlashingV2.Request memory)
    {
        bytes32 slashId = _pendingSlashingRequestIds[service][operator];
        return _slashingRequests[slashId];
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function requestSlashing(Payload calldata payload)
        external
        override
        onlyService(_msgSender())
        whenNotPaused
        returns (bytes32)
    {
        require(bytes(payload.reason).length <= 250, "reason too long");
        require(payload.mbips > 0, "mbips must be > 0");
        require(payload.timestamp <= block.timestamp, "timestamp in future");

        address service = _msgSender();
        ISLAYRegistryV2.SlashParameter memory slashParameter =
            registry.getSlashParameterAt(service, payload.operator, payload.timestamp);

        require(payload.mbips <= slashParameter.maxMbips, "mbips exceeds max allowed");
        require(
            payload.timestamp > (block.timestamp - registry.getWithdrawalDelay(payload.operator)), "timestamp too old"
        );

        bytes32 slashId = _pendingSlashingRequestIds[service][payload.operator];
        if (slashId != bytes32(0)) {
            ISLAYRouterSlashingV2.Request storage pendingRequest = _slashingRequests[slashId];

            if (pendingRequest.status == ISLAYRouterSlashingV2.Status.Locked) {
                revert("Previous slashing request lifecycle not completed");
            }

            if (pendingRequest.status == ISLAYRouterSlashingV2.Status.Pending) {
                if (pendingRequest.requestExpiry > uint32(block.timestamp)) {
                    // The previous slashing request is pending and has not expired
                    revert("Previous slashing request lifecycle not completed");
                } else {
                    // The previous slashing request is pending but expired
                    // eligible for new slashing request to take place
                    // by canceling the previous slashing request.
                    pendingRequest.status = ISLAYRouterSlashingV2.Status.Canceled;
                    emit ISLAYRouterSlashingV2.SlashingCanceled(service, payload.operator, slashId);
                }
            }
        }

        uint32 requestResolution = uint32(block.timestamp) + slashParameter.resolutionWindow;
        ISLAYRouterSlashingV2.Request memory request = ISLAYRouterSlashingV2.Request({
            status: ISLAYRouterSlashingV2.Status.Pending,
            service: service,
            mbips: payload.mbips,
            timestamp: payload.timestamp,
            requestTime: uint32(block.timestamp),
            operator: payload.operator,
            requestResolution: requestResolution,
            requestExpiry: requestResolution + SLASHING_REQUEST_EXPIRY_WINDOW
        });

        slashId = SlashingRequestId.hash(request);
        _pendingSlashingRequestIds[service][payload.operator] = slashId;
        _slashingRequests[slashId] = request;
        emit ISLAYRouterSlashingV2.SlashingRequested(service, payload.operator, slashId, request, payload.reason);
        return slashId;
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function lockSlashing(bytes32 slashId) external override whenNotPaused onlyService(_msgSender()) {
        ISLAYRouterSlashingV2.Request storage request = _slashingRequests[slashId];
        // Only service that initiated the slash request can call this function.
        if (request.service != _msgSender()) {
            revert ISLAYRouterSlashingV2.LockSlashingNotAuthorized();
        }

        // Check if the slashing request is pending.
        if (request.status != ISLAYRouterSlashingV2.Status.Pending) {
            revert ISLAYRouterSlashingV2.LockSlashingStatusIsNotPending();
        }

        // Check if the slashing request has not expired
        if (request.requestExpiry < uint32(block.timestamp)) {
            revert ISLAYRouterSlashingV2.LockSlashingExpired();
        }

        // Check if the slashing request is after the resolution window has passed
        if (request.requestResolution > uint32(block.timestamp)) {
            revert ISLAYRouterSlashingV2.LockSlashingResolutionNotReached();
        }

        // Iterate through the vaults and call lockSlashing on each of them
        EnumerableSet.AddressSet storage operatorVaults = _operatorVaults[request.operator];
        uint256 vaultsCount = operatorVaults.length();
        for (uint256 i = 0; i < vaultsCount;) {
            address vaultAddress = operatorVaults.at(i);
            ISLAYVaultV2 vault = ISLAYVaultV2(vaultAddress);

            // calculate the slash amount from mbips
            uint256 slashAmount = Math.mulDiv(vault.totalAssets(), request.mbips, 10_000_000);

            // Call the lockSlashing function on the vault
            vault.lockSlashing(slashAmount);

            // Store the locked assets in the router for further processing
            _lockedAssets[slashId].push(ISLAYRouterSlashingV2.LockedAssets({amount: slashAmount, vault: vaultAddress}));

            // vaultsCount is bounded to _maxVaultsPerOperator
            unchecked {
                i++;
            }
        }

        // update the slashing request status to Locked
        request.status = ISLAYRouterSlashingV2.Status.Locked;

        emit ISLAYRouterSlashingV2.SlashingLocked(request.service, request.operator, slashId);
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function getLockedAssets(bytes32 slashId)
        external
        view
        override
        returns (ISLAYRouterSlashingV2.LockedAssets[] memory)
    {
        return _lockedAssets[slashId];
    }
}
