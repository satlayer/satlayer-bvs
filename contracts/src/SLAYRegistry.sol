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
    mapping(address service => bool) public services;
    mapping(address operator => bool) public operators;

    using Checkpoints for Checkpoints.Trace224;

    /**
     * @dev Service <-> Operator registration is a two sided consensus
     * This mean both service and operator has to register to pair with each other
     * See [`RegistrationStatus`] enum for more information
     */
    mapping(bytes32 service_operator_hash => Checkpoints.Trace224) private registration_status;

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
         *
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

    /**
     * @dev Register as a service
     * This function allows an address to be registered as a service.
     */
    function registerAsService(string memory uri, string memory name) public {
        require(!services[msg.sender], "Service has been registered");
        services[msg.sender] = true;
        emit ServiceRegistered(msg.sender, uri, name);
    }

    /**
     * @dev Register as a service
     * This function allows an address to be registered as an operator.
     */
    function registerAsOperator(string memory uri, string memory name) public {
        require(!operators[msg.sender], "Operator has been registerd");
        operators[msg.sender] = true;
        emit OperatorRegistered(msg.sender, uri, name);
    }

    /**
     * @dev Register an operator to a service (info.sender is the service)
     * Service must be registered via [`RegisterAsService()`].
     * If the operator has registered this service, the registration status will be set to [`RegistrationStatus.Active`] (1)
     * Else the registration status will be set to [`RegistrationStatus.ServiceRegistered`] (3)
     */
    function registerOperatorToService(address operator) public {
        address service = msg.sender;
        require(operators[operator], "Operator not found");
        require(services[service], "Service not found");

        bytes32 key = _getKey(service, operator);
        RegistrationStatus status = getLatestRegistrationStatus(key);

        if (status == RegistrationStatus.Active) {
            revert("Registration is already active");
        } else if (status == RegistrationStatus.ServiceRegistered) {
            revert("Service has already registered this operator");
        } else if (status == RegistrationStatus.Inactive) {
            setRegistrationStatus(key, RegistrationStatus.ServiceRegistered);

            emit RegistrationStatusUpdated(operator, service, RegistrationStatus.ServiceRegistered);
        } else if (status == RegistrationStatus.OperatorRegistered) {
            setRegistrationStatus(key, RegistrationStatus.Active);

            emit RegistrationStatusUpdated(operator, service, RegistrationStatus.Active);
        } else {
            // should not branch into this.
            revert("Invalid registration state");
        }
    }

    /**
     * @dev Deregister an operator from a service (info.sender is the service)
     * Service must be registered via [`RegisterAsService()`].
     * If the operator is not registered with this service, it will revert.
     * If the operator is registered with this service, it will set the registration status to [`RegistrationStatus.Inactive`] (0)
     */
    function deregisterOperatorFromService(address operator) public {
        address service = msg.sender;

        require(services[service], "Service not registered");

        bytes32 key = _getKey(service, operator);
        RegistrationStatus status = getLatestRegistrationStatus(key);

        if (status == RegistrationStatus.Inactive) {
            revert("Operator is not registered with this service");
        }

        setRegistrationStatus(key, RegistrationStatus.Inactive);

        emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Inactive);
    }

    /**
     * @dev Register a service to an operator (info.sender is the operator)
     * Operator must be registered with [`RegisterAsOperator()`]
     * If the service has registered this operator, the registration status will be set to [`RegistrationStatus::Active`] (1)
     * Else the registration status will be set to [`RegistrationStatus.OperatorRegistered`] (2)
     */
    function registerServiceToOperator(address service) public {
        address operator = msg.sender;
        require(services[service], "Service not registered");
        require(operators[operator], "Operator not registered");

        bytes32 key = _getKey(service, operator);
        RegistrationStatus status = getLatestRegistrationStatus(key);

        if (status == RegistrationStatus.Active) {
            revert("Registration between operator and service is already active");
        } else if (status == RegistrationStatus.OperatorRegistered) {
            revert("Operator has already registered this service");
        } else if (status == RegistrationStatus.Inactive) {
            setRegistrationStatus(key, RegistrationStatus.OperatorRegistered);

            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.OperatorRegistered);
        } else if (status == RegistrationStatus.ServiceRegistered) {
            setRegistrationStatus(key, RegistrationStatus.Active);

            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Active);
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
    function deregisterServiceFromOperator(address service) public {
        address operator = msg.sender;
        require(operators[operator], "Operator not registered");
        require(services[service], "Service not registered to operator");

        bytes32 key = _getKey(service, operator);
        RegistrationStatus status = getLatestRegistrationStatus(key);

        if (status == RegistrationStatus.Inactive) {
            revert("Service is not registered to this operator");
        }

        setRegistrationStatus(key, RegistrationStatus.Inactive);

        emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Inactive);
    }

    /**
     * @dev Hash the service and operator addresses to create a unique key for the registration_status map.
     */
    function _getKey(address service, address operator) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(service, operator));
    }

    /**
     * @dev Get the registration_status for a given service-operator pair at the latest checkpoint.
     * @param key The hash of the service and operator addresses. See `_getKey()`.
     * Returns the latest registration status as an enum value.
     */
    function getLatestRegistrationStatus(bytes32 key) public view returns (RegistrationStatus) {
        // checkpoint.latest() returns 0 on null cases, that nicely fit into
        // RegistrationStatus.Inactive being 0
        return RegistrationStatus(uint8(registration_status[key].latest()));
    }

    /**
     * @dev Get the registration status for a service-operator pair at a specific timestamp.
     * @param key The hash of the service and operator addresses. See `_getKey()`.
     * @param timestamp The timestamp to check the registration status at.
     * Returns the registration status as an enum value.
     */
    function getRegistrationStatusAt(bytes32 key, uint32 timestamp) public view returns (RegistrationStatus) {
        return RegistrationStatus(uint8(registration_status[key].upperLookup(timestamp)));
    }

    /**
     * @dev Set the registration status for a service-operator pair.
     * This function is used internally to update the registration status.
     * @param key The hash of the service and operator addresses. See `_getKey()`.
     * @param status The new registration status to set.
     */
    function setRegistrationStatus(bytes32 key, RegistrationStatus status) internal {
        registration_status[key].push(uint32(block.timestamp), uint224(uint8(status)));
    }

    /**
     * @dev Check if an address is registered as an operator.
     * @param operator The address to check.
     * @return True if the address is registered as an operator, false otherwise.
     */
    function isOperator(address operator) public view returns (bool) {
        return operators[operator];
    }

    /**
     * @dev Check if an address is registered as a service.
     * @param service The address to check.
     * @return True if the address is registered as a service, false otherwise.
     */
    function isService(address service) public view returns (bool) {
        return services[service];
    }
}
