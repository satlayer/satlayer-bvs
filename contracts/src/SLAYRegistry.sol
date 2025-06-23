// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";

import {SLAYRouter} from "./SLAYRouter.sol";

/**
 * @title SLAYRegistry
 * @dev This contract serves as a registry for services and operators in the SatLayer ecosystem.
 * It allows services and operators to register themselves, manage their relationships,
 * and track registration statuses.
 */
contract SLAYRegistry is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
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
     * @dev Stored slashing parameter for each of every slash enabled BVS services.
     */
    mapping(address service => Checkpoints.Trace224) private _slashParameters;

    /**
     * @dev A service may enable slashing but operator will have to opt in to the slashing
     * This map store a list of operator that has opt in to the slashing for particular service
     */
    mapping(bytes32 key => Checkpoints.Trace224) private _slashingOptIns;

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
     * @dev Emitted when Slashing Parameter for a service is updated
     * @param service The address of the service
     * @param destination The address at which slash collateral will be moved.
     * @param maxMilliBip The maximum slashable amount
     * @param resolutionWindow An operator's refutable period in seconds in the event of slash.
     */
    event SlashingParameterUpdated(
        address indexed service, address destination, uint32 maxMilliBip, uint32 resolutionWindow
    );

    /**
     * @dev Emitted when operator is opt into the slashing for particular service
     * @param service The address of the service.
     * @param operator The address of the operator.
     */
    event SlashingOptIn(address indexed service, address indexed operator);

    /**
     * @dev Set the immutable SLAYRouter proxy address for the implementation.
     * Cyclic params in constructor are possible as an EmptyImpl is used for an initial deployment,
     * after which all the contracts are upgraded to their respective implementations with immutable proxy addresses.
     *
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(SLAYRouter router_) {
        router = router_;
        _disableInitializers();
    }

    function initialize() public reinitializer(2) {
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
     * @dev Modifier to guard if given service operator pair is Actively Paired - [`RegistrationStatus.Active`]
     */
    modifier onlyActivelyRegistered(address service, address operator) {
        RegistrationStatus status = getRegistrationStatus(service, operator);
        if (status != RegistrationStatus.Active) {
            revert("RegistrationStatus not Active");
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
     * @dev service enable slashing by providing slashing parameters
     * _msgSender is registered service.
     */
    function enableSlashing(SlashParameter.object calldata parameter) public onlyService(_msgSender()) {
        require(parameter.maxMilliBips < 10000001, "Maximum Milli-Bips cannot be more than 10_000_000 (100%)");
        require(parameter.maxMilliBips > 0, "Minimum Bips cannot be less than zero");
        address service = _msgSender();
        _updateSlashingParameters(service, parameter.destination, parameter.maxMilliBips, parameter.resolutionWindow);
    }

    /**
     * @dev Mutate slash parameter checkpoint state for particular service
     */
    function _updateSlashingParameters(address service, address destination, uint32 maxBips, uint32 resolutionWindow)
        internal
    {
        uint224 parameter = SlashParameter.encode(destination, maxBips, resolutionWindow);
        _slashParameters[service].push(uint32(block.timestamp), parameter);
        emit SlashingParameterUpdated(service, destination, maxBips, resolutionWindow);
    }

    /**
     * @dev Get latest slashing parameters for particular service
     */
    function getSlashingParameter(address service) public view returns (SlashParameter.object memory) {
        SlashParameter.object memory parameter = SlashParameter.decode(_slashParameters[service].latest());
        return parameter;
    }

    /**
     * @dev Get slashing parameters for particular service at or near at a given point in time.
     */
    function getSlashingParameterAt(address service, uint256 timestamp)
        public
        view
        returns (SlashParameter.object memory)
    {
        SlashParameter.object memory parameter =
            SlashParameter.decode(_slashParameters[service].upperLookup(uint32(timestamp)));
        return parameter;
    }

    /**
     * @dev An operator can opt in to slashing for particular service
     * _msgSender() is operator
     */
    function slashingOptIn(address service)
        public
        onlyService(service)
        onlyOperator(_msgSender())
        onlyActivelyRegistered(service, _msgSender())
    {
        address operator = _msgSender();
        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        bool optedIn = getSlashingOptIns(key);
        require(optedIn == false, "Operator already opted in slashing for this service");
        _updateSlashingOptIns(key, true);
        emit SlashingOptIn(service, operator);
    }

    /**
     * @dev Mutate the operator slash opt in map at current block timestamp.
     */
    function _updateSlashingOptIns(bytes32 key, bool optIn) internal {
        _slashingOptIns[key].push(uint32(block.timestamp), uint224(optIn ? 1 : 0));
    }

    /**
     * @dev Get if an operator is opted in to slash for particular service at current block timestamp
     */
    function getSlashingOptIns(bytes32 key) public view returns (bool) {
        bool optedIn = _slashingOptIns[key].latest() == 1 ? true : false;
        return optedIn;
    }

    /**
     * @dev Get if an operator is opted in to slash for particular service at current block timestamp
     */
    function getSlashingOptIns(address service, address operator) public view returns (bool) {
        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        bool optedIn = _slashingOptIns[key].latest() == 1 ? true : false;
        return optedIn;
    }

    /**
     * @dev Get if an operator is opted in to slash
     * for particular service at or near at given timestamp
     */
    function getSlashingOptInsAt(bytes32 key, uint256 timestamp) public view returns (bool) {
        bool optedIn = (_slashingOptIns[key].upperLookup(uint32(timestamp))) == 1 ? true : false;
        return optedIn;
    }

    /**
     * @dev Get if an operator is opted in to slash
     * for particular service at or near at given timestamp
     */
    function getSlashingOptInsAt(address service, address operator, uint256 timestamp) public view returns (bool) {
        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        bool optedIn = (_slashingOptIns[key].upperLookup(uint32(timestamp))) == 1 ? true : false;
        return optedIn;
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

library SlashParameter {
    /**
     * @dev Slash Parameters for particular service
     */
    struct object {
        /**
         * The address at which the slash collateral from the vault
         * will be moved to at the end of slashing lifecycle
         */
        address destination;
        /**
         * The maximum slash amount represented in bips at milli unit.
         * 1 Milli-Bip is 0.00001%
         * At 100% - the milli bip is 10,000,000
         */
        uint32 maxMilliBips;
        /**
         * The time window in seconds at which operator can refute slash accusations.
         * The exact mechanics are to be defined by the BVS (service).
         */
        uint32 resolutionWindow;
    }

    /**
     * @dev Encode [`SlashParatmer.object`] into uint224 to be used as checkpoint value.
     */
    function encode(address destination, uint32 maxBip, uint32 resolutionWindow) internal pure returns (uint224) {
        uint160 addr160 = uint160(destination);

        uint224 encodedData = uint224(addr160);

        encodedData |= (uint224(maxBip) << 160);

        encodedData |= (uint224(resolutionWindow) << 192);

        return encodedData;
    }

    /**
     * @dev Decode uint224 from checkpoint value into [`SlashParatmer.object`].
     */
    function decode(uint224 encodedData) internal pure returns (SlashParameter.object memory) {
        address addr = address(uint160(encodedData));

        uint32 maxBip = uint32(encodedData >> 160);

        uint32 resolutionWindow = uint32(encodedData >> 192);

        return SlashParameter.object({destination: addr, maxMilliBips: maxBip, resolutionWindow: resolutionWindow});
    }
}
