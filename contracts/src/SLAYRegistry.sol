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

    mapping(bytes32 service_operator_hash => Checkpoints.Trace224) private registration_status;

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

    function registerAsService(string memory uri, string memory name) public {
        require(!services[msg.sender], "Service has been registered");
        services[msg.sender] = true;
        emit ServiceRegistered(msg.sender, uri, name);
    }

    function registerAsOperator(string memory uri, string memory name) public {
        require(!operators[msg.sender], "Operator has been registerd");
        operators[msg.sender] = true;
        emit OperatorRegistered(msg.sender, uri, name);
    }

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

    function _getKey(address service, address operator) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(service, operator));
    }

    function getLatestRegistrationStatus(bytes32 key) public view returns (RegistrationStatus) {
        // checkpoint.latest() returns 0 on null cases, that nicely fit into
        // RegistrationStatus.Inactive being 0
        return RegistrationStatus(uint8(registration_status[key].latest()));
    }

    function getRegistrationStatusAt(bytes32 key, uint256 blockNumber) public view returns (RegistrationStatus) {
        return RegistrationStatus(uint8(registration_status[key].upperLookup(uint32(blockNumber))));
    }

    /// @notice Set the status for a service-operator pair at current block
    function setRegistrationStatus(bytes32 key, RegistrationStatus status) internal {
        registration_status[key].push(uint32(block.number), uint224(uint8(status)));
    }
}
