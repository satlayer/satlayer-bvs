// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";

import {SLAYRouter} from "./SLAYRouter.sol";

contract SLAYRegistry is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    SLAYRouter public immutable router;
    mapping(address service => bool) private _services;
    mapping(address operator => bool) private _operators;

    using Checkpoints for Checkpoints.Trace224;

    /**
     * @dev Service <-> Operator registration is a two sided consensus
     * This mean both service and operator has to register to pair with each other
     * See [`RegistrationStatus`] enum for more information
     */
    mapping(bytes32 serviceOperatorHash => Checkpoints.Trace224) private registrationStatus;

    enum RegistrationStatus {
        /**
         * Default state when neither the Operator nor the Service has registered,
         * or when either the Operator or Service has unregistered
         */
        Inactive,
        /**
         * State when both the Operator and Service have registered with each other,
         * indicating a fully established relationship
         */
        Active,
        /**
         * State when only the Operator has registered but the Service hasn't yet registered,
         * indicating a pending registration from the Service side
         * This is Operator-initiated registration, waiting for Service to finalize
         */
        OperatorRegistered,
        /**
         * This state is used when the Service has registered an Operator, but the Operator hasn't yet registered,
         * indicating a pending registration from the Operator side
         * This is Service-initiated registration, waiting for Operator to finalize
         */
        ServiceRegistered
    }

    event ServiceRegistered(address indexed service, string uri, string name);

    event OperatorRegistered(address indexed operator, string uri, string name);

    event RegistrationStatusUpdated(address indexed service, address indexed operator, RegistrationStatus status);

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

    modifier onlyService(address operator) {
        require(_services[_msgSender()], "Only registered service can call this function");
        require(_operators[operator], "The operator being attempted to pair does not exist");
        _;
    }

    modifier onlyOperator(address service) {
        require(_operators[_msgSender()], "Only registered operator can call this function");
        require(_services[service], "The service being attempted to pair does not exist");
        _;
    }

    /**
     * @dev Register as a service
     * This function allows an address to be registered as a service.
     * @param uri uri of the service project.
     * @param name name of the service project.
     */
    function registerAsService(string memory uri, string memory name) public {
        require(!_services[_msgSender()], "Already registered");
        _services[_msgSender()] = true;
        emit ServiceRegistered(_msgSender(), uri, name);
    }

    /**
     * @dev Register as a service
     * This function allows an address to be registered as an operator.
     * @param uri uri of the operator
     * @param name name of the operator
     */
    function registerAsOperator(string memory uri, string memory name) public {
        require(!_operators[_msgSender()], "Already registered");
        _operators[_msgSender()] = true;
        emit OperatorRegistered(_msgSender(), uri, name);
    }

    /**
     * @dev Register an operator to a service (_msgSender() is the service)
     * Service must be registered via [`RegisterAsService()`].
     * If the operator has registered this service, the registration status will be set to [`RegistrationStatus.Active`] (1)
     * Else the registration status will be set to [`RegistrationStatus.ServiceRegistered`] (3)
     */
    function registerOperatorToService(address operator) public onlyService(operator) {
        address service = _msgSender();

        bytes32 key = ServiceOperator._getKey(service, operator);
        RegistrationStatus status = getRegistrationStatus(key);

        if (status == RegistrationStatus.Active) {
            revert("Registration is already active");
        } else if (status == RegistrationStatus.ServiceRegistered) {
            revert("Service has already registered this operator");
        } else if (status == RegistrationStatus.Inactive) {
            _setRegistrationStatus(RegistrationStatus.ServiceRegistered, service, operator, key);
        } else if (status == RegistrationStatus.OperatorRegistered) {
            _setRegistrationStatus(RegistrationStatus.Active, service, operator, key);
        } else {
            // should not branch into this.
            revert("Invalid registration state");
        }
    }

    /**
     * @dev Deregister an operator from a service (_msgSender() is the service)
     * Service must be registered via [`RegisterAsService()`].
     * If the operator is not registered with this service, it will revert.
     * If the operator is registered with this service, it will set the registration status to [`RegistrationStatus.Inactive`] (0)
     */
    function deregisterOperatorFromService(address operator) public onlyService(operator) {
        address service = _msgSender();
        bytes32 key = ServiceOperator._getKey(service, operator);
        RegistrationStatus status = getRegistrationStatus(key);

        if (status == RegistrationStatus.Inactive) {
            revert("Operator is not registered with this service");
        }

        _setRegistrationStatus(RegistrationStatus.Inactive, service, operator, key);
    }

    /**
     * @dev Register a service to an operator (info.sender is the operator)
     * Operator must be registered with [`RegisterAsOperator()`]
     * If the service has registered this operator, the registration status will be set to [`RegistrationStatus::Active`] (1)
     * Else the registration status will be set to [`RegistrationStatus.OperatorRegistered`] (2)
     */
    function registerServiceToOperator(address service) public onlyOperator(service) {
        address operator = _msgSender();

        bytes32 key = ServiceOperator._getKey(service, operator);
        RegistrationStatus status = getRegistrationStatus(key);

        if (status == RegistrationStatus.Active) {
            revert("Registration between operator and service is already active");
        } else if (status == RegistrationStatus.OperatorRegistered) {
            revert("Operator has already registered this service");
        } else if (status == RegistrationStatus.Inactive) {
            _setRegistrationStatus(RegistrationStatus.OperatorRegistered, service, operator, key);
        } else if (status == RegistrationStatus.ServiceRegistered) {
            _setRegistrationStatus(RegistrationStatus.Active, service, operator, key);
        } else {
            revert("Invalid registration state");
        }
    }

    /**
     * @dev Deregister a service from an operator (info.sender is the operator)
     * Operator must be registered with [`RegisterAsOperator()`]
     * If the service is not registered to the operator, it will revert.
     * If the service is registered to the operator, it will set the registration status to [`RegistrationStatus.Inactive`] (0)
     */
    function deregisterServiceFromOperator(address service) public onlyOperator(service) {
        address operator = _msgSender();

        bytes32 key = ServiceOperator._getKey(service, operator);
        RegistrationStatus status = getRegistrationStatus(key);

        if (status == RegistrationStatus.Inactive) {
            revert("Service is not registered to this operator");
        }

        _setRegistrationStatus(RegistrationStatus.Inactive, service, operator, key);
    }

    /**
     * @dev Get the `registrationStatus` for a given service-operator pair at the latest checkpoint.
     * @param key The hash of the service and operator addresses. See `ServiceOperator._getKey()`.
     * Returns the latest registration status as an enum value.
     * Recommended to use for cases where key is already calculated before calling the function.
     * Saves gas for hash computation.
     */
    function getRegistrationStatus(bytes32 key) public view returns (RegistrationStatus) {
        // checkpoint.latest() returns 0 on null cases, that nicely fit into
        // RegistrationStatus.Inactive being 0
        return RegistrationStatus(uint8(registrationStatus[key].latest()));
    }

    /**
     * @dev Get the `registrationStatus` for a given service-operator pair at the latest checkpoint.
     * @param service address of the service for the pair in question.
     * @param operator address of the operator for the pair in question.
     * Returns the latest registration status as an enum value.
     * Exactly the same as its overloaded counter part except key is calculated on the go.
     * Uses more gas.
     */
    function getRegistrationStatus(address service, address operator) public view returns (RegistrationStatus) {
        bytes32 key = ServiceOperator._getKey(service, operator);
        return RegistrationStatus(uint8(registrationStatus[key].latest()));
    }

    /**
     * @dev Get the registration status for a service-operator pair at a specific timestamp.
     * @param key The hash of the service and operator addresses. See `ServiceOperator._getKey()`.
     * @param timestamp The timestamp to check the registration status at.
     * Returns the registration status as an enum value.
     */
    function getRegistrationStatusAt(bytes32 key, uint32 timestamp) public view returns (RegistrationStatus) {
        return RegistrationStatus(uint8(registrationStatus[key].upperLookup(timestamp)));
    }

    /**
     * @dev Get the registration status for a service-operator pair at a specific timestamp.
     * @param service address of the service for the pair in question.
     * @param operator address of the operator for the pair in question.
     * Returns the registration status as an enum value.
     */
    function getRegistrationStatusAt(address service, address operator, uint32 timestamp)
        public
        view
        returns (RegistrationStatus)
    {
        bytes32 key = ServiceOperator._getKey(service, operator);
        return RegistrationStatus(uint8(registrationStatus[key].upperLookup(timestamp)));
    }

    /**
     * @dev Set the registration status for a service-operator pair.
     * @param status RegistrationStatus member.
     * @param service address of the service.
     * @param operator address of the operator.
     * Calculate the hash of service operator pair on the go.
     * Uses more gas.
     */
    function _setRegistrationStatus(RegistrationStatus status, address service, address operator) internal {
        bytes32 key = ServiceOperator._getKey(service, operator);
        registrationStatus[key].push(uint32(block.timestamp), uint224(uint8(status)));

        emit RegistrationStatusUpdated(service, operator, status);
    }

    /**
     * @dev Set the registration status for a service-operator pair.
     * @param status RegistrationStatus member.
     * @param service address of the service.
     * @param operator address of the operator.
     * Recommended to use for cases where key is already calculated before calling the function.
     * Saves gas for hash computation.
     */
    function _setRegistrationStatus(RegistrationStatus status, address service, address operator, bytes32 key)
        internal
    {
        registrationStatus[key].push(uint32(block.timestamp), uint224(uint8(status)));

        emit RegistrationStatusUpdated(service, operator, status);
    }

    /**
     * @dev Check if an address is registered as an operator.
     * @param operator The address to check.
     * @return True if the address is registered as an operator, false otherwise.
     */
    function isOperator(address operator) public view returns (bool) {
        return _operators[operator];
    }

    /**
     * @dev Check if an address is registered as a service.
     * @param service The address to check.
     * @return True if the address is registered as a service, false otherwise.
     */
    function isService(address service) public view returns (bool) {
        return _services[service];
    }
}

library ServiceOperator {
    /**
     * @dev Hash the service and operator addresses to create a unique key for the `registrationStatus` map.
     * @param service address of the service.
     * @param operator address of the operator.
     * returns the hash (bytes32)
     */
    function _getKey(address service, address operator) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(service, operator));
    }
}
