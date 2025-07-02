// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";

import {SLAYRouter} from "./SLAYRouter.sol";
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
    using Checkpoints for Checkpoints.Trace224;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRouter public immutable router;

    /// @dev mapping of registered services.
    mapping(address service => bool) private _services;

    /// @dev mapping of registered operators.
    mapping(address operator => bool) private _operators;

    /// @dev mapping of withdrawal delays for all of operator's vault.
    mapping(address operator => uint32) private _withdrawalDelay;

    /**
     * @dev Service <-> Operator registration is a two sided consensus.
     * This mean both service and operator has to register to pair with each other.
     */
    mapping(bytes32 key => Checkpoints.Trace224) private _registrationStatus;

    /// @dev Default delay for operator's vault withdrawals if not set.
    uint32 public constant DEFAULT_WITHDRAWAL_DELAY = 7 days;

    mapping(bytes32 key => Checkpoints.Trace224) private _slashOptIns;

    /**
     * @dev Store {SlashParameter.Object} for each of every slash enabled BVS services.
     */
    mapping(address service => Checkpoints.Trace224) private _slashParameters;

    /**
     * @dev Store the hash of {SlashParameter.Object} for each of every slash enabled BVS services.
     * When service changed the slashing parameters, operator should explicitly opt in to new parameters.
     * This mapping helps with reseting the operator's slash optin states without having to loop through
     * which could potentially hits gas ceiling.
     */
    mapping(address service => bytes32) private _slashParameterHashes;

    /**
     * @dev Incremented every time {enableSlashing} is called.
     * This prevents implicit slash opt in for cases when
     * service x has parameter a,b,c - all operator opted in
     * then service x updated to f,g,h and finally went back to a,b,c.
     * Without nonce tracking the operator opt in states will be
     * 1.true for original a,b,c params they opted in
     * 2.false for f,g,h params they have not opted in explicitly
     * 3.true for a,b,c params they have not opted in explicitly
     * Nonce prevents same param re-occurance implicit slash opt ins.
     * This is checkpointed to support accurate reading for {getSlashParameterAt()}
     */
    Checkpoints.Trace224 private _slashParameterUpdateNonce;

    /**
     * @dev Emitted when {SlashParameter.Object} for a service is updated
     * @param service The address of the service
     * @param destination The address at which slash collateral will be moved.
     * @param maxMilliBip The maximum slashable amount
     * @param resolutionWindow An operator's refutable period in seconds in the event of slash.
     */
    event SlashParameterUpdated(
        address indexed service, address destination, uint32 maxMilliBip, uint32 resolutionWindow
    );

    /**
     * @dev Emitted when operator is opt into the slashing for particular service
     * @param service The address of the service.
     * @param operator The address of the operator.
     */
    event SlashOptIn(address indexed service, address indexed operator);

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
     * @dev Modifier to guard if given service operator pair is Actively Paired - {RegistrationStatus.Active}.
     */
    modifier onlyActivelyRegistered(address service, address operator) {
        RegistrationStatus status = getRegistrationStatus(service, operator);
        if (status != RegistrationStatus.Active) {
            revert("RegistrationStatus not Active");
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
        address service = _msgSender();

        require(!_services[service], "Already registered");
        _services[service] = true;
        emit ServiceRegistered(service);
        emit MetadataUpdated(service, uri, name);
    }

    /// @inheritdoc ISLAYRegistry
    function registerAsOperator(string memory uri, string memory name) external whenNotPaused {
        address operator = _msgSender();

        require(!_operators[operator], "Already registered");
        _operators[operator] = true;
        emit OperatorRegistered(operator);
        emit MetadataUpdated(operator, uri, name);
    }

    /// @inheritdoc ISLAYRegistry
    function updateMetadata(string memory uri, string memory name) external whenNotPaused {
        address provider = _msgSender();
        require(_services[provider] || _operators[provider], "Not registered");

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

    /// @inheritdoc ISLAYRegistry
    function deregisterOperatorFromService(address operator)
        external
        whenNotPaused
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

    /// @inheritdoc ISLAYRegistry
    function registerServiceToOperator(address service)
        external
        whenNotPaused
        onlyOperator(_msgSender())
        onlyService(service)
    {
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

    /// @inheritdoc ISLAYRegistry
    function deregisterServiceFromOperator(address service)
        external
        whenNotPaused
        onlyOperator(_msgSender())
        onlyService(service)
    {
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
        return RegistrationStatus(_registrationStatus[key].latest());
    }

    /// @inheritdoc ISLAYRegistry
    function getRegistrationStatusAt(address service, address operator, uint32 timestamp)
        public
        view
        returns (RegistrationStatus)
    {
        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        return RegistrationStatus(_registrationStatus[key].upperLookup(timestamp));
    }

    /**
     * @dev Get the `RegistrationStatus` for a given service-operator pair at the latest checkpoint.
     * @param key The hash of the service and operator addresses. Use `ServiceOperator._getKey()` to generate the key.
     * @return RegistrationStatus The latest registration status for the service-operator pair.
     */
    function _getRegistrationStatus(bytes32 key) internal view returns (RegistrationStatus) {
        // The method `checkpoint.latest()` returns 0 on empty checkpoint,
        // RegistrationStatus.Inactive being 0 as desired.
        return RegistrationStatus(_registrationStatus[key].latest());
    }

    /**
     * @dev Set the registration status for a service-operator pair.
     * @param key The hash of the service and operator addresses. Use `ServiceOperator._getKey()` to generate the key.
     * @param status RegistrationStatus to set for the service-operator pair.
     */
    function _updateRegistrationStatus(bytes32 key, RegistrationStatus status) internal {
        _registrationStatus[key].push(uint32(block.timestamp), uint224(status));
    }

    /// @inheritdoc ISLAYRegistry
    function isOperator(address account) external view returns (bool) {
        return _operators[account];
    }

    /// @inheritdoc ISLAYRegistry
    function isService(address account) external view returns (bool) {
        return _services[account];
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

    /// @dev Pauses the contract.
    function pause() external onlyOwner {
        _pause();
    }

    /// @dev Unpauses the contract.
    function unpause() external onlyOwner {
        _unpause();
    }

    /**
     * @dev service enable slashing by providing slash parameters in the form of {SlashParameter.Object}.
     * _msgSender is registered service.
     */
    function enableSlashing(SlashParameter.Object calldata parameter) public onlyService(_msgSender()) {
        require(parameter.maxMilliBips <= 10000000, "Maximum Milli-Bips cannot be more than 10_000_000 (100%)");
        require(parameter.maxMilliBips >= 0, "Minimum Bips cannot be less than zero");
        _updateSlashParameter(_msgSender(), parameter.destination, parameter.maxMilliBips, parameter.resolutionWindow);
    }

    /**
     * @dev service disable slashing by giving soldity primitive types' null default
     * _msgSender is registered service.
     */
    function disableSlashing() public onlyService(_msgSender()) {
        _updateSlashParameter(_msgSender(), address(0), uint32(0), uint32(0));
    }

    /**
     * @dev Mutate slash parameter checkpoint state for particular service
     */
    function _updateSlashParameter(address service, address destination, uint32 maxMilliBips, uint32 resolutionWindow)
        internal
    {
        uint224 parameter = SlashParameter.encode(destination, maxMilliBips, resolutionWindow);
        _slashParameters[service].push(uint32(block.timestamp), parameter);

        _slashParameterHashes[service] = SlashParameter._hashParameters(
            service, destination, maxMilliBips, resolutionWindow, _getSlashParameterNonce() + 1
        );
        _incrementSlashParameterNonce();
        emit SlashParameterUpdated(service, destination, maxMilliBips, resolutionWindow);
    }

    /**
     * @dev Get latest {SlashParameter.Object} for particular service
     */
    function getSlashParameter(address service) public view returns (SlashParameter.Object memory) {
        SlashParameter.Object memory parameter = SlashParameter.decode(_slashParameters[service].latest());
        return parameter;
    }

    /**
     * @dev Get slashing parameters for particular service at or near at a given point in time.
     */
    function getSlashParameterAt(address service, uint32 timestamp)
        public
        view
        returns (SlashParameter.Object memory)
    {
        SlashParameter.Object memory parameter = SlashParameter.decode(_slashParameters[service].upperLookup(timestamp));
        return parameter;
    }

    /**
     * @dev Increment slash parameter nonce at latest timestamp
     */
    function _incrementSlashParameterNonce() internal {
        uint224 previousNonce = _getSlashParameterNonce();
        _slashParameterUpdateNonce.push(uint32(block.timestamp), (previousNonce + 1));
    }

    /**
     * @dev get slash parameter nonce at latest timestamp
     */
    function _getSlashParameterNonce() internal view returns (uint224) {
        return _slashParameterUpdateNonce.latest();
    }

    /**
     * @dev get slash parameter nonce at given timestamp
     */
    function _getSlashParameterNonceAt(uint32 timestamp) internal view returns (uint224) {
        return _slashParameterUpdateNonce.upperLookup(timestamp);
    }

    /**
     * @dev An operator can opt in to slashing for particular service
     * _msgSender() is operator
     */
    function slashOptIn(address service) public onlyActivelyRegistered(service, _msgSender()) {
        require(_slashParameterHashes[service] != bytes32(0), "Service disabled slashing");
        address operator = _msgSender();
        bool optedIn = getSlashOptIns(service, operator);
        require(optedIn == false, "Operator already opted in slashing for this service");
        _updateSlashOptIns(service, operator, true);
        emit SlashOptIn(service, operator);
    }

    /**
     * @dev Mutate the operator slash opt in map at current block timestamp.
     */
    function _updateSlashOptIns(address service, address operator, bool optIn) internal {
        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        if (optIn == true) {
            // truncating bytes32 hash into uint224.
            _slashOptIns[key].push(uint32(block.timestamp), uint224(uint256(_slashParameterHashes[service]) >> (8 * 4)));
        } else {
            _slashOptIns[key].push(uint32(block.timestamp), uint224(0));
        }
    }

    /**
     * @dev Get if an operator is opted in to slash for particular service at current block timestamp
     */
    function getSlashOptIns(address service, address operator) public view returns (bool) {
        bytes32 key = ServiceOperatorKey._getKey(service, operator);
        uint224 truncated = uint224(uint256(_slashParameterHashes[service]) >> (8 * 4));
        uint224 optedIn = _slashOptIns[key].latest();

        if (optedIn == truncated) {
            return true;
        }

        return false;
    }

    /**
     * @dev Get if an operator is opted in to slash
     * for particular service at or near at given timestamp
     */
    function getSlashOptInsAt(address service, address operator, uint32 timestamp) public view returns (bool) {
        bytes32 key = ServiceOperatorKey._getKey(service, operator);

        uint224 operatorOptedInTruncatedHash = _slashOptIns[key].upperLookup(timestamp);

        if (operatorOptedInTruncatedHash == 0) {
            return false;
        }

        SlashParameter.Object memory serviceParamsAtTimestamp =
            SlashParameter.decode(_slashParameters[service].upperLookup(timestamp));

        bytes32 fullHashOfServiceParamsAtTimestamp = SlashParameter._hashParameters(
            service,
            serviceParamsAtTimestamp.destination,
            serviceParamsAtTimestamp.maxMilliBips,
            serviceParamsAtTimestamp.resolutionWindow,
            _getSlashParameterNonceAt(timestamp)
        );
        uint224 serviceTruncatedHashAtTimestamp = uint224(uint256(fullHashOfServiceParamsAtTimestamp) >> (8 * 4));

        return operatorOptedInTruncatedHash == serviceTruncatedHashAtTimestamp;
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
    struct Object {
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
     * @dev Encode {SlashParameter.Object} into uint224 to be used as checkpoint value.
     * Each data in the struct combined is still smaller than uint224.
     * Dedicating checkpoints for each shard of data is expensive.
     * Encoding the data in sequence avoid uncessary cold SLOAD, especially each member in the struct
     * are access together usually.
     */
    function encode(address destination, uint32 maxBip, uint32 resolutionWindow) internal pure returns (uint224) {
        uint160 addr160 = uint160(destination);

        uint224 encodedData = uint224(addr160);

        encodedData |= (uint224(maxBip) << 160);

        encodedData |= (uint224(resolutionWindow) << 192);

        return encodedData;
    }

    /**
     * @dev Decode uint224 from checkpoint value into {SlashParatmer.Object}.
     */
    function decode(uint224 encodedData) internal pure returns (SlashParameter.Object memory) {
        address addr = address(uint160(encodedData));

        uint32 maxBip = uint32(encodedData >> 160);

        uint32 resolutionWindow = uint32(encodedData >> 192);

        return SlashParameter.Object({destination: addr, maxMilliBips: maxBip, resolutionWindow: resolutionWindow});
    }

    /**
     * @dev Hash the slashing parameter for particular service.
     * The hash include the nonce to avoid replay/implicit opted in for case like
     * same service, same slash param re-occurance.
     * Read more at {_slashParameterUpdateNonce}.
     */
    function _hashParameters(
        address service,
        address destination,
        uint32 maxMilliBips,
        uint32 resolutionWindow,
        uint256 nonce
    ) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(service, destination, maxMilliBips, resolutionWindow, nonce));
    }
}
