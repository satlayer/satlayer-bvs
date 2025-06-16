// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import {SLAYRouter} from "./SLAYRouter.sol";

contract SLAYRegistry is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    SLAYRouter public immutable router;
    mapping(address => bool) public services;
    mapping(address => bool) public operators;
    mapping(byte32 => bool) public registration_status;

    enum RegistrationStatus {
        /// Default state when neither the Operator nor the Service has registered,
        /// or when either the Operator or Service has unregistered
        Inactive,
        /// State when both the Operator and Service have registered with each other,
        /// indicating a fully established relationship
        Active,
        /// State when only the Operator has registered but the Service hasn't yet registered,
        /// indicating a pending registration from the Service side
        /// This is Operator-initiated registration, waiting for Service to finalize
        OperatorRegistered,
        /// State when only the Service has registered but the Operator hasn't yet registered,
        /// indicating a pending registration from the Operator side
        /// This is Service-initiated registration, waiting for Operator to finalize
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

    function registerAsService(string uri, string name) public {
        require(!services[msg.sender], "Service has been registered");
        services[msg.sender] = true;
        emit ServiceRegistered(msg.sender, uri, name);
    }

    function registerAsOperator(string uri, string name) public {
        require(!operators[msg.sender], "Operator has been registerd");
        operators[msg.sender] = true;
        emit OperatorRegistered(msg.sender, uri, name);
    }

    function registerOperatorToService(address operator) public {
        address service = msg.sender;
        require(operators[operator], "Operator not found");
        require(services[service], "Service not found");

        byte32 key = _getKey(service, operator);
        RegistrationStatus status = registration_status[key];

        if (status == RegistrationStatus.Active) {
            revert("Registration is already active");
        } else if (status == RegistrationStatus.ServiceRegistered) {
            revert("Service has already registered this operator");
        } else if (status == RegistrationStatus.Inactive) {
            registration_status[key] = RegistrationStatus.ServiceRegistered;

            emit RegistrationStatusUpdated(operator, service, RegistrationStatus.ServiceRegistered);
        } else if (status == RegistrationStatus.OperatorRegistered) {
            registration_status[key] = RegistrationStatus.Active;

            emit RegistrationStatusUpdated(operator, service, RegistrationStatus.Active);
        } else {
            revert("Invalid registration state");
        }
    }

    function deregisterOperatorFromService(address operator) public {
        address service = msg.sender;

        require(services[service], "Service not registered");

        bytes32 key = _getKey(operator, service);
        RegistrationStatus status = registration_status[key];

        if (status == RegistrationStatus.Inactive) {
            revert("Operator is not registered with this service");
        }

        registration_status[key] = RegistrationStatus.Inactive;

        emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Inactive);
    }

    function registerServiceToOperator(address service) public {
        address operator = msg.sender;
        require(services[service], "Service not registered");
        require(operators[operator], "Operator not registered");

        bytes32 key = _getKey(operator, service);
        RegistrationStatus status = registration_status[key];

        if (status == RegistrationStatus.Active) {
            revert("Registration between operator and service is already active");
        } else if (status == RegistrationStatus.OperatorRegistered) {
            revert("Operator has already registered this service");
        } else if (status == RegistrationStatus.Inactive) {
            registration_status[key] = RegistrationStatus.OperatorRegistered;

            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.OperatorRegistered);
        } else if (status == RegistrationStatus.ServiceRegistered) {
            registration_status[key] = RegistrationStatus.Active;

            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Active);
        } else {
            // treat default (None) same as Inactive?
            registration_status[key] = RegistrationStatus.OperatorRegistered;

            emit RegistrationStatusUpdated(service, operator, RegistrationStatus.OperatorRegistered);
        }
    }

    function deregisterServiceFromOperator(address service) public {
        address operator = msg.sender;
        require(operators[operator], "Operator not registered");
        require(service[service], "Service not registered to operator");

        byte32 key = _getKey(service, operator);
        RegistrationStatus status = registration_status[key];

        if (status == RegistrationStatus.Inactive) {
            revert("Service is not registered to this operator");
        }

        registration_status[key] = RegistrationStatus.Inactive;

        emit RegistrationStatusUpdated(service, operator, RegistrationStatus.Inactive);
    }

    function _getKey(address service, address operator) internal pure returns (byte32) {
        return keccak256(abi.encodePacked(service, operator));
    }
}
