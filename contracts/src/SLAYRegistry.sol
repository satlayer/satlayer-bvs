// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";

import {SLAYRouter} from "./SLAYRouter.sol";
import {Relationship} from "./Relationship.sol";
import {ISLAYRegistry} from "./interface/ISLAYRegistry.sol";

/**
 * @title SLAYRegistry
 * @dev This contract serves as a registry for services and operators in the SatLayer ecosystem.
 * It allows services and operators to register themselves, manage their relationships,
 * and track registration statuses.
 *
 * @custom:oz-upgrades-from src/InitialImpl.sol:InitialImpl
 */
contract SLAYRegistry is ISLAYRegistry, Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRouter public immutable router;

    /// @dev mapping of registered services.
    mapping(address account => Service) private _services;

    /// @dev mapping of registered operators.
    mapping(address account => Operator) private _operators;

    /// @dev Slash parameters for services created by the service when {enableSlashing(SlashParameter)} is enabled.
    SlashParameter[] private _slashParameters;

    /**
     * @dev Service <-> Operator registration is a two sided consensus.
     * This mean both service and operator has to register to pair with each other.
     */
    mapping(bytes32 key => Checkpoints.Trace224) private _relationships;

    /// @dev mapping of withdrawal delays for all of operator's vault.
    /// TODO(k): move to within Operator struct?
    mapping(address operator => uint32) private _withdrawalDelay;

    /// @dev Default delay for operator's vault withdrawals if not set.
    uint32 public constant DEFAULT_WITHDRAWAL_DELAY = 7 days;

    /**
     * @dev Initializes SLAYRegistry contract.
     * Set up slash parameters array to allow the first service to register with a valid ID.
     * As `0` is considered as "no slashing enabled" and is used to disable slashing.
     * Instead of using offset, this is cleaner and less prone to errors.
     */
    function initialize() public reinitializer(2) {
        // Push an empty slash parameter to the array to ensure that the first service can register with a valid ID.
        _slashParameters.push();
    }

    /**
     * @dev Modifier to check if the provided account is a registered service.
     * Reverts with `ServiceNotFound` if the account is not registered as a service.
     */
    modifier onlyService(address account) {
        if (!_services[account].registered) {
            revert ServiceNotFound(account);
        }
        _;
    }

    /**
     * @dev Modifier to check if the provided account is a registered operator.
     * Reverts with `OperatorNotFound` if the account is not registered as an operator.
     */
    modifier onlyOperator(address account) {
        if (!_operators[account].registered) {
            revert OperatorNotFound(account);
        }
        _;
    }

    /**
     * @dev Set the immutable SLAYRouter proxy address for the implementation.
     * Cyclic params in constructor are possible as an InitialImpl (empty implementation) is used for an initial deployment,
     * after which all the contracts are upgraded to their respective implementations with immutable proxy addresses.
     *
     * InitialImpl.initialize() is called to set the initial owner of the contract.
     * No other initialization is required for this implementation contract.
     *
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(SLAYRouter router_) {
        router = router_;
        _disableInitializers();
    }

    /**
     * @dev Authorizes an upgrade to a new implementation.
     * This function is required by UUPS and restricts upgradeability to the contract owner.
     * @param newImplementation The address of the new contract implementation.
     */
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    /// @inheritdoc ISLAYRegistry
    function registerAsService(string memory uri, string memory name) external whenNotPaused {
        address account = _msgSender();
        Service storage service = _services[account];

        require(!service.registered, "Already registered");
        service.registered = true;
        emit ServiceRegistered(account);
        emit MetadataUpdated(account, uri, name);
    }

    /// @inheritdoc ISLAYRegistry
    function registerAsOperator(string memory uri, string memory name) external whenNotPaused {
        address account = _msgSender();
        Operator storage operator = _operators[account];

        require(!operator.registered, "Already registered");
        operator.registered = true;
        emit OperatorRegistered(account);
        emit MetadataUpdated(account, uri, name);
    }

    /// @inheritdoc ISLAYRegistry
    function updateMetadata(string memory uri, string memory name) external whenNotPaused {
        address provider = _msgSender();
        require(_services[provider].registered || _operators[provider].registered, "Not registered");

        emit MetadataUpdated(provider, uri, name);
    }

    /// @inheritdoc ISLAYRegistry
    function registerOperatorToService(address operator)
        external
        whenNotPaused
        onlyService(_msgSender())
        onlyOperator(operator)
    {
        address service = _msgSender();
        Relationship.Object memory obj = _getRelationshipObject(service, operator);

        if (obj.status == Relationship.Status.Active) {
            revert("Already active");
        } else if (obj.status == Relationship.Status.ServiceRegistered) {
            revert("Already initiated");
        } else if (obj.status == Relationship.Status.Inactive) {
            obj.status = Relationship.Status.ServiceRegistered;
        } else if (obj.status == Relationship.Status.OperatorRegistered) {
            obj.status = Relationship.Status.Active;
        } else {
            // Panic as this is not an expected state.
            revert("Invalid status");
        }
        _updateRelationshipObject(service, operator, obj);
    }

    /// @inheritdoc ISLAYRegistry
    function deregisterOperatorFromService(address operator)
        external
        onlyService(_msgSender())
        onlyOperator(operator)
    {
        address service = _msgSender();
        Relationship.Object memory obj = _getRelationshipObject(service, operator);

        if (obj.status == Relationship.Status.Inactive) {
            revert("Already inactive");
        }

        _updateRelationshipObject(
            service, operator, Relationship.Object({status: Relationship.Status.Inactive, slashParameterId: 0})
        );
    }

    /// @inheritdoc ISLAYRegistry
    function registerServiceToOperator(address service) external onlyOperator(_msgSender()) onlyService(service) {
        address operator = _msgSender();
        Relationship.Object memory obj = _getRelationshipObject(service, operator);

        if (obj.status == Relationship.Status.Active) {
            revert("Already active");
        } else if (obj.status == Relationship.Status.OperatorRegistered) {
            revert("Already initiated");
        } else if (obj.status == Relationship.Status.Inactive) {
            obj.status = Relationship.Status.OperatorRegistered;
        } else if (obj.status == Relationship.Status.ServiceRegistered) {
            obj.status = Relationship.Status.Active;
        } else {
            // Panic as this is not an expected state.
            revert("Invalid status");
        }
        _updateRelationshipObject(service, operator, obj);
    }

    /// @inheritdoc ISLAYRegistry
    function deregisterServiceFromOperator(address service) external onlyOperator(_msgSender()) onlyService(service) {
        address operator = _msgSender();
        Relationship.Object memory obj = _getRelationshipObject(service, operator);

        if (obj.status == Relationship.Status.Inactive) {
            revert("Already inactive");
        }

        _updateRelationshipObject(
            service, operator, Relationship.Object({status: Relationship.Status.Inactive, slashParameterId: 0})
        );
    }

    /// @inheritdoc ISLAYRegistry
    function getRelationshipStatus(address service, address operator) external view returns (Relationship.Status) {
        Relationship.Object memory obj = _getRelationshipObject(service, operator);
        return obj.status;
    }

    /// @inheritdoc ISLAYRegistry
    function getRelationshipStatusAt(address service, address operator, uint32 timestamp)
        external
        view
        returns (Relationship.Status)
    {
        Relationship.Object memory obj = _getRelationshipObjectAt(service, operator, timestamp);
        return obj.status;
    }

    /// @inheritdoc ISLAYRegistry
    function isOperator(address account) external view returns (bool) {
        return _operators[account].registered;
    }

    /// @inheritdoc ISLAYRegistry
    function isService(address account) external view returns (bool) {
        return _services[account].registered;
    }

    /// @inheritdoc ISLAYRegistry
    function setWithdrawalDelay(uint32 delay) public whenNotPaused onlyOperator(_msgSender()) {
        require(delay >= DEFAULT_WITHDRAWAL_DELAY, "Delay must be at least more than or equal to 7 days");
        _withdrawalDelay[_msgSender()] = delay;
        emit WithdrawalDelayUpdated(_msgSender(), delay);
    }

    /// @inheritdoc ISLAYRegistry
    function getWithdrawalDelay(address operator) public view returns (uint32) {
        // If the delay is not set, return the default delay.
        uint32 delay = _withdrawalDelay[operator];
        return delay == 0 ? DEFAULT_WITHDRAWAL_DELAY : delay;
    }

    /// @inheritdoc ISLAYRegistry
    function enableSlashing(SlashParameter calldata parameter) external onlyService(_msgSender()) whenNotPaused {
        require(parameter.destination != address(0), "destination!=0");
        require(parameter.maxMbips <= 10_000_000, "maxMbips!=>10000000");
        require(parameter.maxMbips > 0, "maxMbips!=0");

        uint256 length = _slashParameters.length;
        require(length <= type(uint32).max, "Overflow");
        _slashParameters.push(parameter);

        address account = _msgSender();
        Service storage service = _services[account];
        service.slashParameterId = uint32(length);
        emit SlashParameterUpdated(account, parameter.destination, parameter.maxMbips, parameter.resolutionWindow);
    }

    /// @inheritdoc ISLAYRegistry
    function disableSlashing() external onlyService(_msgSender()) whenNotPaused {
        address account = _msgSender();
        Service storage service = _services[account];
        // 0 is used to indicate that slashing is disabled.
        service.slashParameterId = 0;
        emit SlashParameterUpdated(account, address(0), 0, 0);
    }

    /// @inheritdoc ISLAYRegistry
    function enableSlashing(address service) external onlyOperator(_msgSender()) whenNotPaused {
        address operator = _msgSender();
        Relationship.Object memory obj = _getRelationshipObject(service, operator);
        require(obj.status == Relationship.Status.Active, "Relationship not active");

        uint32 slashParameterId = _services[service].slashParameterId;
        require(slashParameterId != 0, "Slashing not enabled");
        require(slashParameterId != obj.slashParameterId, "Same slashing parameters");
        obj.slashParameterId = slashParameterId;
        _updateRelationshipObject(service, operator, obj);
    }

    /// @inheritdoc ISLAYRegistry
    function getSlashParameter(address service) external view returns (SlashParameter memory) {
        uint32 slashParameterId = _services[service].slashParameterId;
        require(slashParameterId > 0, "Slashing not enabled");
        return _slashParameters[slashParameterId];
    }

    /// @dev Pauses the contract.
    function pause() external onlyOwner {
        _pause();
    }

    /// @dev Unpauses the contract.
    function unpause() external onlyOwner {
        _unpause();
    }

    /**
     * @dev Retrieves the relationship object for a given service-operator pair at a specific timestamp.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @param timestamp The timestamp at which to retrieve the relationship status.
     * @return Relationship.Object The relationship object containing status and other details at the specified timestamp.
     */
    function _getRelationshipObjectAt(address service, address operator, uint32 timestamp)
        internal
        view
        returns (Relationship.Object memory)
    {
        bytes32 key = Relationship.getKey(service, operator);
        return Relationship.upperLookup(_relationships[key], timestamp);
    }

    /**
     * @dev Retrieves the latest relationship object for a given service-operator pair.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @return Relationship.Object The latest relationship object containing status and other details.
     */
    function _getRelationshipObject(address service, address operator)
        internal
        view
        returns (Relationship.Object memory)
    {
        bytes32 key = Relationship.getKey(service, operator);
        return Relationship.latest(_relationships[key]);
    }

    /**
     * @dev Updates the relationship status for a given service-operator pair.
     * We require the {service} and {operator} addresses to be passed in as parameters,
     * instead of using a pre-computed relationship {key} to emit the event and ensure proper usage of the function.
     * @param service The address of the service.
     * @param operator The address of the operator.
     * @param obj The relationship object containing the new status and other details.
     */
    function _updateRelationshipObject(address service, address operator, Relationship.Object memory obj)
        internal
        whenNotPaused
    {
        bytes32 key = Relationship.getKey(service, operator);
        Relationship.push(_relationships[key], uint32(block.timestamp), obj);

        // if the status is active, increment the relationships count for both service and operator.
        // If the status is inactive, decrement the relationships count for both service and operator.
        if (obj.status == Relationship.Status.Active) {
            Operator storage operator = _operators[operator];
            if (operator.activeServicesCount >= Relationship.MAX_ACTIVE_RELATIONSHIPS()) {
                revert ISLAYRegistry.OperatorRelationshipsExceeded();
            }
            Service storage service = _services[service];
            if (service.activeOperatorsCount >= Relationship.MAX_ACTIVE_RELATIONSHIPS()) {
                revert ISLAYRegistry.ServiceRelationshipsExceeded();
            }

            // using unchecked for gas optimization as we already checked the counts above.
            unchecked {
                operator.activeServicesCount++;
                service.activeOperatorsCount++;
            }
        } else if (obj.status == Relationship.Status.Inactive) {
            Operator storage operator = _operators[operator];
            Service storage service = _services[service];

            unchecked {
                if (operator.activeServicesCount != 0) {
                    operator.activeServicesCount--;
                }
                if (service.activeOperatorsCount != 0) {
                    service.activeOperatorsCount--;
                }
            }
        }

        emit RelationshipUpdated(service, operator, obj.status, obj.slashParameterId);
    }
}
