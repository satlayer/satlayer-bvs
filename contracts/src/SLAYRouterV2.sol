// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {SLAYRegistryV2} from "./SLAYRegistryV2.sol";
import {ISLAYRegistryV2} from "./interface/ISLAYRegistryV2.sol";
import {ISLAYRouterV2, Slashing} from "./interface/ISLAYRouterV2.sol";
import {RelationshipV2} from "./RelationshipV2.sol";
import {ISLAYVaultV2} from "./interface/ISlAYVaultV2.sol";

/**
 * @title Vaults Router Contract
 * @dev The central point for managing interactions with SLAYVaults.
 * This contract is designed to work with the SLAYRegistryV2 for managing vaults and their states.
 *
 * @custom:oz-upgrades-from src/InitialImpl.sol:InitialImpl
 */
contract SLAYRouterV2 is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable, ISLAYRouterV2 {
    using EnumerableSet for EnumerableSet.AddressSet;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    ISLAYRegistryV2 public immutable registry;

    /// @dev Whitelisted flag for each vault.
    mapping(address => bool) internal _whitelisted;
    /**
     * @notice 7 days
     */
    uint32 public constant slashingRequestExpiryWindow = 7 days;

    mapping(address => bool) public whitelisted;

    /// @dev The max number of vaults allowed per operator.
    uint8 private _maxVaultsPerOperator;

    /// @dev Stores the EnumerableSet of vault address for each operator.
    mapping(address operator => EnumerableSet.AddressSet) private _operatorVaults;

    /// @dev Stores the id for most recent slashing request for a given service operator pair.
    mapping(bytes32 serviceOperatorKey => bytes32 slashId) public slashingRequestIds;

    /// @dev Stores the slashing requests by its id.
    mapping(bytes32 slashId => Slashing.RequestInfo) public slashingRequests;

    modifier onlyValidSlashRequest(Slashing.RequestPayload memory request) {
        ISLAYRegistryV2.SlashParameter memory slashBounds =
            registry.getSlashParameterAt(_msgSender(), request.operator, request.timestamp);

        require(
            request.millieBips <= slashBounds.maxMbips, "Slash requested amount is more than the service has allowed"
        );
        require(request.millieBips > 0, "Requested slashing amount in milli bips must be greater than zero");

        uint32 withdrawalDelay = registry.getWithdrawalDelay(request.operator);

        require(
            request.timestamp > (block.timestamp - withdrawalDelay),
            "Slash timestamp must be within the allowable slash period"
        );

        require(request.timestamp <= block.timestamp, "Cannot request slash with timestamp greater than present");
        _;
    }

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

    /// @inheritdoc ISLAYRouterV2
    function requestSlashing(Slashing.RequestPayload memory payload) external onlyValidSlashRequest(payload) {
        address service = _msgSender();
        Slashing.RequestInfo memory pendingSlashingRequest = getPendingSlashingRequest(service, payload.operator);

        if (Slashing.isRequestInfoExist(pendingSlashingRequest) == true) {
            if (pendingSlashingRequest.status == Slashing.RequestStatus.Pending) {
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

            if (pendingSlashingRequest.status == Slashing.RequestStatus.Locked) {
                revert("Previous slashing request lifecycle not completed");
            }
        }

        uint32 requestResolution = uint32(block.timestamp)
            + registry.getSlashParameterAt(service, payload.operator, payload.timestamp).resolutionWindow;
        uint32 requestExpiry = requestResolution + slashingRequestExpiryWindow;

        Slashing.RequestInfo memory newSlashingRequestInfo = Slashing.RequestInfo({
            request: payload,
            requestTime: uint32(block.timestamp),
            requestResolution: requestResolution,
            requestExpiry: requestExpiry,
            status: Slashing.RequestStatus.Pending,
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
        returns (Slashing.RequestInfo memory)
    {
        bytes32 key = RelationshipV2.getKey(service, operator);
        bytes32 slashId = slashingRequestIds[key];
        return slashingRequests[slashId];
    }

    /**
     * Create a new slashing request
     * @param service Address of the service.
     * @param operator Address of the operator.
     * @param info {ISLAYRouter.Slashing.RequestInfo}
     */
    function _createNewSlashingRequest(address service, address operator, Slashing.RequestInfo memory info)
        internal
        returns (bytes32)
    {
        bytes32 key = RelationshipV2.getKey(service, operator);
        bytes32 slashId = Slashing.calculateSlashingRequestId(info);
        slashingRequestIds[key] = slashId;
        slashingRequests[slashId] = info;
        emit ISLAYRouterV2.NewSlashingRequest(service, operator, slashId, info);
        return slashId;
    }

    /**
     * Cancel Slashing request
     * @param service Address of the service.
     * @param operator Address of the operator.
     * @param pendingSlashingRequest {ISLAYRouter.Slashing.RequestInfo}
     */
    function _cancelSlashingRequest(
        address service,
        address operator,
        Slashing.RequestInfo memory pendingSlashingRequest
    ) internal {
        pendingSlashingRequest.status = Slashing.RequestStatus.Canceled;
        bytes32 slashId = Slashing.calculateSlashingRequestId(pendingSlashingRequest);
        bytes32 key = RelationshipV2.getKey(service, operator);

        delete slashingRequestIds[key];

        slashingRequests[slashId] = pendingSlashingRequest;
        emit ISLAYRouterV2.CancelSlashingRequest(service, operator, slashId, pendingSlashingRequest);
    }
}
