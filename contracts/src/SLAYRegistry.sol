// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";

import {SLAYRouter} from "./SLAYRouter.sol";

/// @title SLAYRegistry
/// @dev This contract serves as a registry for services and operators in the SatLayer ecosystem.
/// It allows services and operators to register themselves, manage their relationships,
/// and track registration statuses.
///
/// @custom:oz-upgrades-from src/InitialImpl.sol:InitialImpl
contract SLAYRegistry is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRouter public immutable router;

    /**
     * @dev mapping of registered services.
     */
    mapping(address service => bool) private _services;
    /**
     * @dev mapping of registered operators.
     */
    mapping(address operator => bool) private _operators;

    /**
     * @dev mapping of withdrawal delays for all of operator's vault.
     */
    mapping(address operator => uint32) private _withdrawalDelay;

    /**
     * @dev Default delay for operator's vault withdrawals if not set.
     */
    uint32 public constant DEFAULT_WITHDRAWAL_DELAY = 7 days;

    /**
     * @dev Account is not registered as an operator.
     */
    error OperatorNotFound(address account);

    /**
     * @dev Account is not registered as a service.
     */
    error ServiceNotFound(address account);

    using Checkpoints for Checkpoints.Trace224;

    /**
     * @dev Service <-> Operator registration is a two sided consensus.
     * This mean both service and operator has to register to pair with each other.
     */
    mapping(bytes32 key => Checkpoints.Trace224) private _registrationStatus;

    /**
     * @dev Enum representing the registration status between a service and an operator.
     * The registration status can be one of the following:
     */
    enum RegistrationStatus {
        /**
         * Default state when neither the Operator nor the Service has registered,
         * or when either the Operator or Service has unregistered.
         * `uint8(0)` is used to represent this state, the default value.
         */
        Inactive,
        /**
         * State when both the Operator and Service have registered with each other,
         * indicating a fully established relationship.
         */
        Active,
        /**
         * This state is used when the Operator has registered an Service,
         * but the Service hasn't yet registered,
         * indicating a pending registration from the Service side.
         * This is Operator-initiated registration, waiting for Service to finalize.
         */
        OperatorRegistered,
        /**
         * This state is used when the Service has registered an Operator,
         * but the Operator hasn't yet registered,
         * indicating a pending registration from the Operator side.
         * This is Service-initiated registration, waiting for Operator to finalize.
         */
        ServiceRegistered
    }

    /**
     * @dev Emitted when a service is registered.
     */
    event ServiceRegistered(address indexed service);

    /**
     * @dev Emitted when a operator is registered.
     */
    event OperatorRegistered(address indexed operator);

    /**
     * @dev Emitted when a service is registered with metadata.
     * Name and URI are not validated or stored on-chain.
     *
     * @param provider The address of the service/operator provider.
     * @param uri URI of the provider's project to display in the UI.
     * @param name Name of the provider's project to display in the UI.
     */
    event MetadataUpdated(address indexed provider, string uri, string name);

    /**
     * @dev Emitted when a service-operator registration status is updated.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @param status The new registration status.
     */
    event RegistrationStatusUpdated(address indexed service, address indexed operator, RegistrationStatus status);

    /**
     * @dev Emitted when an operator updates the withdrawal delay.
     * @param operator The address of the operator setting the delay.
     * @param delay The new withdrawal delay in seconds.
     */
    event WithdrawalDelayUpdated(address indexed operator, uint32 delay);

    /**
     * @dev Set the immutable SLAYRouter proxy address for the implementation.
     * Cyclic params in constructor are possible as an InitialImpl (empty implementation) is used for an initial deployment,
     * after which all the contracts are upgraded to their respective implementations with immutable proxy addresses.
     *
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(SLAYRouter router_) {
        router = router_;
        _disableInitializers();
    }

    /// @custom:oz-upgrades-validate-as-initializer
    function initialize(address initialOwner) public reinitializer(2) {
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
        __Pausable_init();
    }

    function initialize2() public reinitializer(2) {
        __Pausable_init();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    /**
     * @dev Modifier to check if the provided account is a registered service.
     * Reverts with `ServiceNotFound` if the account is not registered as a service.
     */
    modifier onlyService(address account) {
        if (!_services[account]) {
            revert ServiceNotFound(account);
        }
        _;
    }

    /**
     * @dev Modifier to check if the provided account is a registered operator.
     * Reverts with `OperatorNotFound` if the account is not registered as an operator.
     */
    modifier onlyOperator(address account) {
        if (!_operators[account]) {
            revert OperatorNotFound(account);
        }
        _;
    }

    /**
     * Register the caller as an service provider.
     * URI and name are not stored on-chain, they're emitted in an event {MetadataUpdated} and separately indexed.
     * The caller can both be a service and an operator. This relationship is not exclusive.
     *
     * @param uri URI of the service's project to display in the UI.
     * @param name Name of the service's project to display in the UI.
     */
    function registerAsService(string memory uri, string memory name) external {
        address service = _msgSender();

        require(!_services[service], "Already registered");
        _services[service] = true;
        emit ServiceRegistered(service);
        emit MetadataUpdated(service, uri, name);
    }

    /**
     * Register the caller as an operator.
     * URI and name are not stored on-chain, they're emitted in an event {MetadataUpdated} and separately indexed.
     * The caller can both be a service and an operator. This relationship is not exclusive.
     *
     * @param uri URI of the operator's project to display in the UI.
     * @param name Name of the operator's project to display in the UI.
     */
    function registerAsOperator(string memory uri, string memory name) external {
        address operator = _msgSender();

        require(!_operators[operator], "Already registered");
        _operators[operator] = true;
        emit OperatorRegistered(operator);
        emit MetadataUpdated(operator, uri, name);
    }

    /**
     * @dev Update metadata for the service or operator.
     * This function can be called by both services and operators.
     * Emits a `MetadataUpdated` event with the new URI and name.
     *
     * Name and URI are not validated or stored on-chain.
     *
     * @param uri URI of the provider's project to display in the UI.
     * @param name Name of the provider's project to display in the UI.
     */
    function updateMetadata(string memory uri, string memory name) external {
        address provider = _msgSender();
        require(_services[provider] || _operators[provider], "Not registered");

        emit MetadataUpdated(provider, uri, name);
    }

    /**
     * @dev To register an operator to a service (the caller is the service).
     * @param operator address of the operator to pair with the service.
     *
     * To call this function, the following conditions must be met:
     *  - Service must be registered via {registerAsService}
     *  - Operator must be registered via {registerAsOperator}
     *
     * If the operator has registered this service, the registration status will be set to `RegistrationStatus.Active`.
     * Else the registration status will be set to `RegistrationStatus.ServiceRegistered`.
     */
    function registerOperatorToService(address operator) external onlyService(_msgSender()) onlyOperator(operator) {
        address service = _msgSender();

        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        RegistrationStatus status = _getRegistrationStatus(key);

        if (status == RegistrationStatus.Active) {
            revert("Already active");
        } else if (status == RegistrationStatus.ServiceRegistered) {
            revert("Already initiated");
        } else if (status == RegistrationStatus.Inactive) {
            _updateRegistrationStatus(key, RegistrationStatus.ServiceRegistered);
            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.ServiceRegistered);
        } else if (status == RegistrationStatus.OperatorRegistered) {
            _updateRegistrationStatus(key, RegistrationStatus.Active);
            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Active);
        } else {
            // Panic as this is not an expected state.
            revert("Invalid status");
        }
    }

    /**
     * @dev Deregister an operator from a service (the caller is the service).
     * @param operator address of the operator to opt out of the relationship.
     */
    function deregisterOperatorFromService(address operator)
        external
        onlyService(_msgSender())
        onlyOperator(operator)
    {
        address service = _msgSender();

        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        if (_getRegistrationStatus(key) == RegistrationStatus.Inactive) {
            revert("Already inactive");
        }

        _updateRegistrationStatus(key, RegistrationStatus.Inactive);
        emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Inactive);
    }

    /**
     * @dev To register an service to a operator (the caller is the operator).
     * @param service address of the service to pair with the operator.
     *
     * To call this function, the following conditions must be met:
     *  - Service must be registered via {registerAsService}
     *  - Operator must be registered via {registerAsOperator}
     *
     * If the service has registered this service, the registration status will be set to `RegistrationStatus.Active`.
     * Else the registration status will be set to `RegistrationStatus.OperatorRegistered`.
     */
    function registerServiceToOperator(address service) external onlyOperator(_msgSender()) onlyService(service) {
        address operator = _msgSender();

        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        RegistrationStatus status = _getRegistrationStatus(key);

        if (status == RegistrationStatus.Active) {
            revert("Already active");
        } else if (status == RegistrationStatus.OperatorRegistered) {
            revert("Already initiated");
        } else if (status == RegistrationStatus.Inactive) {
            _updateRegistrationStatus(key, RegistrationStatus.OperatorRegistered);
            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.OperatorRegistered);
        } else if (status == RegistrationStatus.ServiceRegistered) {
            _updateRegistrationStatus(key, RegistrationStatus.Active);
            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Active);
        } else {
            // Panic as this is not an expected state.
            revert("Invalid status");
        }
    }

    /**
     * @dev Deregister an service from a operator (the caller is the operator).
     * @param service address of the service to opt out of the relationship.
     */
    function deregisterServiceFromOperator(address service) external onlyOperator(_msgSender()) onlyService(service) {
        address operator = _msgSender();

        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        if (_getRegistrationStatus(key) == RegistrationStatus.Inactive) {
            revert("Already inactive");
        }

        _updateRegistrationStatus(key, RegistrationStatus.Inactive);
        emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Inactive);
    }

    /**
     * @dev Get the `RegistrationStatus` for a given service-operator pair at the latest checkpoint.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @return RegistrationStatus The latest registration status for the service-operator pair.
     */
    function getRegistrationStatus(address service, address operator) public view returns (RegistrationStatus) {
        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        return RegistrationStatus(uint8(_registrationStatus[key].latest()));
    }

    /**
     * @dev Get the `RegistrationStatus` for a given service-operator pair at a specific timestamp.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @return RegistrationStatus The registration status at the specified timestamp.
     */
    function getRegistrationStatusAt(address service, address operator, uint32 timestamp)
        public
        view
        returns (RegistrationStatus)
    {
        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        return RegistrationStatus(uint8(_registrationStatus[key].upperLookup(timestamp)));
    }

    /**
     * @dev Get the `RegistrationStatus` for a given service-operator pair at the latest checkpoint.
     * @param key The hash of the service and operator addresses. Use `ServiceOperator._getKey()` to generate the key.
     * @return RegistrationStatus The latest registration status for the service-operator pair.
     */
    function _getRegistrationStatus(bytes32 key) internal view returns (RegistrationStatus) {
        // The method `checkpoint.latest()` returns 0 on empty checkpoint,
        // RegistrationStatus.Inactive being 0 as desired.
        return RegistrationStatus(uint8(_registrationStatus[key].latest()));
    }

    /**
     * @dev Set the registration status for a service-operator pair.
     * @param key The hash of the service and operator addresses. Use `ServiceOperator._getKey()` to generate the key.
     * @param status RegistrationStatus to set for the service-operator pair.
     */
    function _updateRegistrationStatus(bytes32 key, RegistrationStatus status) internal {
        _registrationStatus[key].push(uint32(block.timestamp), uint224(uint8(status)));
    }

    /**
     * Check if an account is registered as a operator.
     * @param account The address to check.
     * @return True if the address is registered as an operator, false otherwise.
     */
    function isOperator(address account) public view returns (bool) {
        return _operators[account];
    }

    /**
     * Check if an address is registered as a service.
     * @param account The address to check.
     * @return True if the address is registered as a service, false otherwise.
     */
    function isService(address account) public view returns (bool) {
        return _services[account];
    }

    /**
     * @notice Set the withdrawal delay for an operator's vault. Only the operator can set this value.
     * @param delay The delay in seconds before a withdrawal can be processed.
     */
    function setWithdrawalDelay(uint32 delay) public onlyOperator(_msgSender()) {
        require(delay >= DEFAULT_WITHDRAWAL_DELAY, "Delay must be at least more than or equal to 7 days");
        _withdrawalDelay[_msgSender()] = delay;
        emit WithdrawalDelayUpdated(_msgSender(), delay);
    }

    /**
     * @notice Get the withdrawal delay for an operator's vault.
     * @dev If the delay is not set, it returns the default delay of 7 days.
     * @param operator The address of the operator.
     * @return uint32 The withdrawal delay in seconds.
     */
    function getWithdrawalDelay(address operator) public view returns (uint32) {
        // If the delay is not set, return the default delay.
        uint32 delay = _withdrawalDelay[operator];
        return delay == 0 ? DEFAULT_WITHDRAWAL_DELAY : delay;
    }
}

library ServiceOperatorKey {
    /**
     * @dev Hash the service and operator addresses to create a unique key for the `registrationStatus` map.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @return bytes32 The unique key for the service-operator pair.
     */
    function _getKey(address service, address operator) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(service, operator));
    }
}
