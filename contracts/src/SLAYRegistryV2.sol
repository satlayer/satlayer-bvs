// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {SLAYBase} from "./SLAYBase.sol";

import {RelationshipV2} from "./RelationshipV2.sol";
import {ISLAYRegistryV2} from "./interface/ISLAYRegistryV2.sol";
import {ISLAYRouterV2} from "./interface/ISLAYRouterV2.sol";

/**
 * @title Services and Operators Registry Contract
 * @notice This contract serves as a registry for services and operators in the SatLayer ecosystem
 * @dev Implements the ISLAYRegistryV2 interface to provide functionality for:
 * - Service and operator registration
 * - Relationship management between services and operators
 * - Slashing parameter configuration and approval
 * - Withdrawal delay settings
 * - Active relationship limits
 *
 * @custom:oz-upgrades-from src/SLAYBase.sol:SLAYBase
 */
contract SLAYRegistryV2 is SLAYBase, ISLAYRegistryV2 {
    using EnumerableSet for EnumerableSet.AddressSet;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    ISLAYRouterV2 public immutable ROUTER;

    /// @dev mapping of registered services.
    mapping(address account => ServiceEntry) private _services;

    /// @dev mapping of registered operators.
    mapping(address account => OperatorEntry) private _operators;

    /// @dev Slash parameters for services created by the service when {enableSlashing(SlashParameter)} is enabled.
    SlashParameter[] private _slashParameters;

    /**
     * @dev Service <-> Operator registration is a two sided consensus.
     * This mean both service and operator has to register to pair with each other.
     */
    mapping(bytes32 key => Checkpoints.Trace224) private _relationships;

    /// @dev mapping of active relationships for an operator. Key is operator, value is a set of active service addresses.
    mapping(address operator => EnumerableSet.AddressSet) private _operatorsActiveRelationships;

    /// @dev mapping of active relationships for a service. Key is service, value is a set of active operator addresses.
    mapping(address service => EnumerableSet.AddressSet) private _servicesActiveRelationships;

    /// @dev Default delay for operator's vault withdrawals if not set.
    uint32 public constant DEFAULT_WITHDRAWAL_DELAY = 7 days;

    /// @dev Returns the maximum number of active relationships allowed for a service or operator.
    uint8 private _maxActiveRelationships;

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
     * @notice Constructor that sets the immutable SLAYRouterV2 proxy address
     * @dev Cyclic parameters in the constructor are possible because an SLAYBase (initial base implementation)
     * is used for the initial deployment, after which all contracts are upgraded to their respective
     * implementations with immutable proxy addresses.
     *
     * This contract extends SLAYBase, which provides the initial owner and pause functionality.
     * SLAYBase.initialize() is called to set the initial owner of the contract.
     *
     * @param router_ The address of the SLAYRouterV2 proxy
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(ISLAYRouterV2 router_) {
        ROUTER = router_;
        _disableInitializers();
    }

    /**
     * @notice Initializes the SLAYRegistryV2 contract with default values
     * @dev Sets up the slash parameters array to allow the first service to register with a valid ID.
     * Since `0` is considered as "no slashing enabled" and is used to disable slashing,
     * we push an empty slash parameter to the array as the first element.
     * This approach is cleaner and less prone to errors than using an offset.
     *
     * Also sets the default maximum active relationships to 5.
     *
     * This function can only be called once (and MUST BE CALLED during upgrade),
     * it is protected by the `reinitializer` modifier.
     */
    function initialize2() public reinitializer(2) {
        // Push an empty slash parameter to the array to ensure that the first service can register with a valid ID.
        _slashParameters.push();
        // Default max active relationships is set to 5.
        _maxActiveRelationships = 5;
    }

    /// @inheritdoc ISLAYRegistryV2
    function registerAsService(string calldata uri, string calldata name) external override whenNotPaused {
        address service = _msgSender();
        ServiceEntry storage serviceEntry = _services[service];

        require(!serviceEntry.registered, "Already registered");
        serviceEntry.registered = true;
        emit ServiceRegistered(service);
        emit MetadataUpdated(service, uri, name);
    }

    /// @inheritdoc ISLAYRegistryV2
    function registerAsOperator(string calldata uri, string calldata name) external override whenNotPaused {
        address operator = _msgSender();
        OperatorEntry storage operatorEntry = _operators[operator];

        require(!operatorEntry.registered, "Already registered");
        operatorEntry.registered = true;
        // Set the default withdrawal delay for the operator.
        operatorEntry.withdrawalDelay = DEFAULT_WITHDRAWAL_DELAY;
        emit OperatorRegistered(operator);
        emit MetadataUpdated(operator, uri, name);
    }

    /// @inheritdoc ISLAYRegistryV2
    function updateMetadata(string calldata uri, string calldata name) external override whenNotPaused {
        address provider = _msgSender();
        // Only registered service or operator can update metadata.
        require(_services[provider].registered || _operators[provider].registered, "Not registered");

        emit MetadataUpdated(provider, uri, name);
    }

    /// @inheritdoc ISLAYRegistryV2
    function registerOperatorToService(address operator)
        external
        override
        onlyService(_msgSender())
        onlyOperator(operator)
    {
        address service = _msgSender();
        RelationshipV2.Object memory obj = _getRelationshipObject(service, operator);

        if (obj.status == RelationshipV2.Status.Active) {
            revert("Already active");
        } else if (obj.status == RelationshipV2.Status.ServiceRegistered) {
            revert("Already initiated");
        } else if (obj.status == RelationshipV2.Status.Inactive) {
            obj.status = RelationshipV2.Status.ServiceRegistered;
        } else if (obj.status == RelationshipV2.Status.OperatorRegistered) {
            obj.status = RelationshipV2.Status.Active;
            obj.slashParameterId = _services[service].slashParameterId;
        } else {
            // Panic as this is not an expected state.
            revert("Invalid status");
        }
        _updateRelationshipObject(service, operator, obj);
    }

    /// @inheritdoc ISLAYRegistryV2
    function deregisterOperatorFromService(address operator)
        external
        override
        onlyService(_msgSender())
        onlyOperator(operator)
    {
        address service = _msgSender();
        RelationshipV2.Object memory obj = _getRelationshipObject(service, operator);

        if (obj.status == RelationshipV2.Status.Inactive) {
            revert("Already inactive");
        }

        _updateRelationshipObject(
            service, operator, RelationshipV2.Object({status: RelationshipV2.Status.Inactive, slashParameterId: 0})
        );
    }

    /// @inheritdoc ISLAYRegistryV2
    function registerServiceToOperator(address service)
        external
        override
        onlyOperator(_msgSender())
        onlyService(service)
    {
        address operator = _msgSender();
        RelationshipV2.Object memory obj = _getRelationshipObject(service, operator);

        if (obj.status == RelationshipV2.Status.Active) {
            revert("Already active");
        } else if (obj.status == RelationshipV2.Status.OperatorRegistered) {
            revert("Already initiated");
        } else if (obj.status == RelationshipV2.Status.Inactive) {
            obj.status = RelationshipV2.Status.OperatorRegistered;
        } else if (obj.status == RelationshipV2.Status.ServiceRegistered) {
            obj.status = RelationshipV2.Status.Active;
            obj.slashParameterId = _services[service].slashParameterId;
        } else {
            // Panic as this is not an expected state.
            revert("Invalid status");
        }
        _updateRelationshipObject(service, operator, obj);
    }

    /// @inheritdoc ISLAYRegistryV2
    function deregisterServiceFromOperator(address service)
        external
        override
        onlyOperator(_msgSender())
        onlyService(service)
    {
        address operator = _msgSender();
        RelationshipV2.Object memory obj = _getRelationshipObject(service, operator);

        if (obj.status == RelationshipV2.Status.Inactive) {
            revert("Already inactive");
        }

        _updateRelationshipObject(
            service, operator, RelationshipV2.Object({status: RelationshipV2.Status.Inactive, slashParameterId: 0})
        );
    }

    /// @inheritdoc ISLAYRegistryV2
    function getRelationshipStatus(address service, address operator)
        external
        view
        override
        returns (RelationshipV2.Status)
    {
        RelationshipV2.Object memory obj = _getRelationshipObject(service, operator);
        return obj.status;
    }

    /// @inheritdoc ISLAYRegistryV2
    function getRelationshipStatusAt(address service, address operator, uint32 timestamp)
        external
        view
        override
        returns (RelationshipV2.Status)
    {
        RelationshipV2.Object memory obj = _getRelationshipObjectAt(service, operator, timestamp);
        return obj.status;
    }

    /// @inheritdoc ISLAYRegistryV2
    function isOperator(address account) external view override returns (bool) {
        return _operators[account].registered;
    }

    /// @inheritdoc ISLAYRegistryV2
    function isService(address account) external view override returns (bool) {
        return _services[account].registered;
    }

    /// @inheritdoc ISLAYRegistryV2
    function setWithdrawalDelay(uint32 delay) external override whenNotPaused onlyOperator(_msgSender()) {
        require(delay >= DEFAULT_WITHDRAWAL_DELAY, "Delay must be at least more than or equal to 7 days");

        // check for all active services of the operator if their minimum withdrawal delay is less than the new delay.
        EnumerableSet.AddressSet storage activeServices = _operatorsActiveRelationships[_msgSender()];
        uint256 activeServicesCount = activeServices.length();
        for (uint256 i = 0; i < activeServicesCount;) {
            address service = activeServices.at(i);
            require(
                _services[service].minWithdrawalDelay <= delay,
                "Operator withdrawal delay must be more than or equal to active service's minimum withdrawal delay"
            );

            // unchecked because we are iterating over a fixed length array, not more than {_maxActiveRelationships}.
            unchecked {
                ++i;
            }
        }

        // update the withdrawal delay for the operator.
        _operators[_msgSender()].withdrawalDelay = delay;

        emit WithdrawalDelayUpdated(_msgSender(), delay);
    }

    /// @inheritdoc ISLAYRegistryV2
    function getWithdrawalDelay(address operator) external view override returns (uint32) {
        return _operators[operator].withdrawalDelay;
    }

    /// @inheritdoc ISLAYRegistryV2
    function enableSlashing(SlashParameter calldata parameter)
        external
        override
        onlyService(_msgSender())
        whenNotPaused
    {
        require(parameter.destination != address(0), "destination!=0");
        require(parameter.maxMbips <= 10_000_000, "maxMbips!=>10000000");
        require(parameter.maxMbips > 0, "maxMbips!=0");

        uint256 length = _slashParameters.length;
        require(length <= type(uint32).max, "Overflow");
        _slashParameters.push(parameter);

        address service = _msgSender();
        ServiceEntry storage serviceEntry = _services[service];
        serviceEntry.slashParameterId = uint32(length);
        emit SlashParameterUpdated(service, parameter.destination, parameter.maxMbips, parameter.resolutionWindow);
    }

    /// @inheritdoc ISLAYRegistryV2
    function disableSlashing() external override onlyService(_msgSender()) whenNotPaused {
        address service = _msgSender();
        ServiceEntry storage serviceEntry = _services[service];
        // 0 is used to indicate that slashing is disabled.
        serviceEntry.slashParameterId = 0;
        emit SlashParameterUpdated(service, address(0), 0, 0);
    }

    /// @inheritdoc ISLAYRegistryV2
    function approveSlashingFor(address service) external override onlyOperator(_msgSender()) whenNotPaused {
        address operator = _msgSender();
        RelationshipV2.Object memory obj = _getRelationshipObject(service, operator);
        // don't need onlyService(service) above since it can only be Active if it's registered.
        require(obj.status == RelationshipV2.Status.Active, "Relationship not active");

        uint32 slashParameterId = _services[service].slashParameterId;
        require(slashParameterId != obj.slashParameterId, "Slashing not updated");
        obj.slashParameterId = slashParameterId;
        _updateRelationshipObject(service, operator, obj);
    }

    /// @inheritdoc ISLAYRegistryV2
    function getSlashParameter(address service) external view override returns (SlashParameter memory) {
        uint32 slashParameterId = _services[service].slashParameterId;
        require(slashParameterId > 0, "Slashing not enabled");
        return _slashParameters[slashParameterId];
    }

    /// @inheritdoc ISLAYRegistryV2
    function getSlashParameterAt(address service, address operator, uint32 timestamp)
        external
        view
        override
        returns (SlashParameter memory)
    {
        uint32 slashParameterId = _getRelationshipObjectAt(service, operator, timestamp).slashParameterId;
        require(slashParameterId > 0, "Slashing not enabled");
        return _slashParameters[slashParameterId];
    }

    /**
     * @dev Retrieves the relationship object for a given service-operator pair at a specific timestamp
     * Uses the Checkpoints library to look up the relationship status at the specified timestamp
     *
     * @param service The address of the service
     * @param operator The address of the operator
     * @param timestamp The timestamp at which to retrieve the relationship status
     * @return The relationship object containing status and slashing parameters at the specified timestamp
     */
    function _getRelationshipObjectAt(address service, address operator, uint32 timestamp)
        internal
        view
        returns (RelationshipV2.Object memory)
    {
        bytes32 key = RelationshipV2.getKey(service, operator);
        return RelationshipV2.upperLookup(_relationships[key], timestamp);
    }

    /**
     * @dev Retrieves the latest relationship object for a given service-operator pair
     * Uses the Checkpoints library to get the most recent relationship status
     *
     * @param service The address of the service
     * @param operator The address of the operator
     * @return The latest relationship object containing status and slashing parameters
     */
    function _getRelationshipObject(address service, address operator)
        internal
        view
        returns (RelationshipV2.Object memory)
    {
        bytes32 key = RelationshipV2.getKey(service, operator);
        return RelationshipV2.latest(_relationships[key]);
    }

    /**
     * @dev Updates the relationship status for a given service-operator pair
     * This function handles the relationship status changes and manages the active relationship sets.
     * We require the {service} and {operator} addresses to be passed in as parameters,
     * instead of using a pre-computed relationship {key} to emit the event and ensure proper usage of the function.
     *
     * If the status is set to Active:
     * - Checks if maximum active relationships would be exceeded
     * - Adds the service to the operator's active relationships and vice versa
     *
     * If the status is set to Inactive:
     * - Removes the service from the operator's active relationships and vice versa
     *
     * @param service The address of the service
     * @param operator The address of the operator
     * @param obj The relationship object containing the new status and slashing parameters
     */
    function _updateRelationshipObject(address service, address operator, RelationshipV2.Object memory obj)
        internal
        whenNotPaused
    {
        bytes32 key = RelationshipV2.getKey(service, operator);
        RelationshipV2.push(_relationships[key], uint32(block.timestamp), obj);

        // If the status is active, add the service to the operator's active relationships and vice versa.
        // If the status is inactive, remove the service from the operator's active relationships and vice versa.
        if (obj.status == RelationshipV2.Status.Active) {
            if (_operatorsActiveRelationships[operator].length() >= _maxActiveRelationships) {
                revert ISLAYRegistryV2.OperatorRelationshipsExceeded();
            }
            if (_servicesActiveRelationships[service].length() >= _maxActiveRelationships) {
                revert ISLAYRegistryV2.ServiceRelationshipsExceeded();
            }

            _operatorsActiveRelationships[operator].add(service);
            _servicesActiveRelationships[service].add(operator);
        } else if (obj.status == RelationshipV2.Status.Inactive) {
            _operatorsActiveRelationships[operator].remove(service);
            _servicesActiveRelationships[service].remove(operator);
        }

        emit RelationshipUpdated(service, operator, obj.status, obj.slashParameterId);
    }

    /// @inheritdoc ISLAYRegistryV2
    function setMaxActiveRelationships(uint8 max) external override onlyOwner {
        require(max > 0, "Max active relationships must be greater than 0");
        uint8 oldMax = _maxActiveRelationships;
        require(max > oldMax, "Max active relationships must be greater than current");
        _maxActiveRelationships = max;
        emit MaxActiveRelationshipsUpdated(oldMax, max);
    }

    /// @inheritdoc ISLAYRegistryV2
    function getMaxActiveRelationships() external view override returns (uint8) {
        return _maxActiveRelationships;
    }

    /// @inheritdoc ISLAYRegistryV2
    function setMinWithdrawalDelay(uint32 delay) external override whenNotPaused onlyService(_msgSender()) {
        require(delay > 0, "Delay must be more than 0");
        address service = _msgSender();

        // checks for each of its active operators if their withdrawal delay is less than the new minimum delay
        EnumerableSet.AddressSet storage activeOperators = _servicesActiveRelationships[service];
        uint256 activeOperatorsCount = activeOperators.length();
        for (uint256 i = 0; i < activeOperatorsCount;) {
            require(
                _operators[activeOperators.at(i)].withdrawalDelay >= delay,
                "Service's minimum withdrawal delay must be less than or equal to active operator's withdrawal delay"
            );

            // unchecked because we are iterating over a fixed length array, not more than {_maxActiveRelationships}.
            unchecked {
                ++i;
            }
        }

        // If all checks pass, set the new minimum delay for the service
        _services[service].minWithdrawalDelay = delay;

        emit MinWithdrawalDelayUpdated(service, delay);
    }

    /// @inheritdoc ISLAYRegistryV2
    function getMinWithdrawalDelay(address service) external view override returns (uint32) {
        return _services[service].minWithdrawalDelay;
    }
}
