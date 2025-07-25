// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";

import {SLAYBase} from "./SLAYBase.sol";

import {ISLAYRegistryV2} from "./interface/ISLAYRegistryV2.sol";
import {ISLAYRouterV2} from "./interface/ISLAYRouterV2.sol";
import {ISLAYVaultV2} from "./interface/ISLAYVaultV2.sol";
import {ISLAYRouterSlashingV2, SlashingRequestId} from "./interface/ISLAYRouterSlashingV2.sol";

/**
 * @title SLAYRouterV2 - Vaults Router Contract
 * @dev The central point for managing interactions with SLAYVaults in the SatLayer protocol.
 *
 * This contract serves as the main interface for services to interact with operator vaults,
 * providing functionality for:
 * - Managing vault whitelisting
 * - Enforcing limits on vaults per operator
 * - Handling the slashing process for operator penalties
 * - Managing guardrail approvals for security
 *
 * The router works closely with the SLAYRegistry for service and operator registration
 * information, and interacts with SLAYVaults to manage assets during slashing operations.
 *
 * @custom:oz-upgrades-from src/SLAYBase.sol:SLAYBase
 */
contract SLAYRouterV2 is SLAYBase, ReentrancyGuardUpgradeable, ISLAYRouterV2, ISLAYRouterSlashingV2 {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SafeERC20 for IERC20;

    /**
     * @notice The expiry window for slashing requests, set to 7 days.
     * @dev After the resolution window has passed, services have this additional time period
     * to lock and finalize a slashing request before it expires and becomes invalid.
     */
    uint32 public constant SLASHING_REQUEST_EXPIRY_WINDOW = 7 days;

    /**
     * @dev This is an immutable reference to the SLAYRegistryV2 contract, set during construction.
     * The router uses this to verify service status and retrieve slashing parameters.
     * @custom:oz-upgrades-unsafe-allow state-variable-immutable
     */
    ISLAYRegistryV2 public immutable REGISTRY;

    /**
     * @dev Mapping that stores the whitelist status for each vault address.
     * When a vault is whitelisted (true), it can be interacted with through the router.
     */
    mapping(address => bool) internal _whitelisted;

    /**
     * @dev The maximum number of vaults allowed per operator.
     * This limit prevents a single operator from controlling too many vaults.
     * Default value is set to 10 during initialization.
     */
    uint8 private _maxVaultsPerOperator;

    /**
     * @dev The address of the guardrail contract that provides additional security
     * by approving or rejecting slashing requests before they can be finalized.
     */
    address private _guardrail;

    /**
     * @dev Mapping that stores the set of vault addresses for each operator.
     * Uses EnumerableSet for efficient iteration and membership checking.
     */
    mapping(address operator => EnumerableSet.AddressSet) private _operatorVaults;

    /**
     * @dev Mapping that stores the ID of the most recent slashing request for each service-operator pair.
     * Used to enforce the rule that a service can only have one active slashing request per operator.
     */
    mapping(address service => mapping(address operator => bytes32)) private _pendingSlashingRequestIds;

    /**
     * @dev Mapping that stores the complete slashing request information for each slashing ID.
     * Contains all the details about a slashing request, including its current status.
     * Historical requests are kept.
     */
    mapping(bytes32 slashId => ISLAYRouterSlashingV2.Request) private _slashingRequests;

    /**
     * @dev Mapping that stores the locked assets for each slashing request.
     * When a slashing request is locked, the assets from each vault are recorded here
     * before being transferred to the final destination during finalization.
     */
    mapping(bytes32 slashId => ISLAYRouterSlashingV2.LockedAssets[]) private _lockedAssets;

    /**
     * @dev Mapping that stores the guardrail approval status for each slashing request.
     * Values: 0 - unset (default), 1 - approved, 2 - rejected.
     * A slashing request can only be finalized if it has been approved by the guardrail.
     */
    mapping(bytes32 slashId => uint8) private _guardrailApproval;

    /**
     * @dev Modifier that restricts function access to registered services only.
     * Verifies that the provided account is a registered service by checking with the registry.
     * Reverts with ServiceNotFound if the account is not a registered service.
     *
     * @param account The address to check if it is a registered service.
     */
    modifier onlyService(address account) {
        if (!REGISTRY.isService(account)) {
            revert ISLAYRegistryV2.ServiceNotFound(account);
        }
        _;
    }

    /**
     * @notice Constructor that sets the registry contract address.
     * @dev Sets the immutable SLAYRegistryV2 proxy address for the implementation.
     *
     * This contract is designed to work with the OpenZeppelin upgradeable contracts pattern.
     * Cyclic parameters in the constructor are possible because SLAYBase (the initial base implementation)
     * is used for the initial deployment, after which all contracts are upgraded to their
     * respective implementations with immutable proxy addresses.
     *
     * This contract extends SLAYBase, which provides the initial owner and pause functionality.
     * SLAYBase.initialize() is called separately to set the initial owner of the contract.
     *
     * @param registry_ The address of the SLAYRegistryV2 contract to use for service and operator information.
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(ISLAYRegistryV2 registry_) {
        REGISTRY = registry_;
        _disableInitializers();
    }

    /**
     * @dev Allows initialization of the contract without having SLAYBase initially initialized.
     * This will run initialize from SLAYBase and then initialize2, if any initialization were previously done,
     * it will revert with an error using the `initializer` modifier/protection.
     *
     * @custom:oz-upgrades-validate-as-initializer
     */
    function initializeAll(address initialOwner) public {
        SLAYBase.initialize(initialOwner);
        initialize2();
    }

    /**
     * @dev This function is called during the upgrade from SLAYBase to SLAYRouterV2.
     * It sets the default maximum number of vaults per operator to 10.
     *
     * This function can only be called once (and MUST BE CALLED during upgrade),
     * it is protected by the `reinitializer` modifier.
     */
    function initialize2() public reinitializer(2) {
        __ReentrancyGuard_init();
        _maxVaultsPerOperator = 10;
    }

    /// @inheritdoc ISLAYRouterV2
    function getMaxVaultsPerOperator() external view override returns (uint8) {
        return _maxVaultsPerOperator;
    }

    /// @inheritdoc ISLAYRouterV2
    function setMaxVaultsPerOperator(uint8 count) external override onlyOwner {
        require(count > _maxVaultsPerOperator, "Must be greater than current");
        _maxVaultsPerOperator = count;
    }

    /// @inheritdoc ISLAYRouterV2
    function setVaultWhitelist(address vault_, bool isWhitelisted) external override onlyOwner {
        require(_whitelisted[vault_] != isWhitelisted, "Vault already in desired state");

        // Get the operator address from the vault contract
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
    function isVaultWhitelisted(address vault_) external view override returns (bool) {
        return _whitelisted[vault_];
    }

    /// @inheritdoc ISLAYRouterV2
    function setGuardrail(address guardrail) external override onlyOwner {
        require(guardrail != address(0), "Guardrail address cannot be empty");
        _guardrail = guardrail;
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function getPendingSlashingRequest(address service, address operator)
        external
        view
        override
        returns (ISLAYRouterSlashingV2.Request memory)
    {
        // Retrieve the current slashing request ID for the given service-operator pair
        bytes32 slashId = _pendingSlashingRequestIds[service][operator];
        return _slashingRequests[slashId];
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function getSlashingRequest(bytes32 slashId)
        external
        view
        override
        returns (ISLAYRouterSlashingV2.Request memory)
    {
        // Return the complete slashing request information for the given slashId
        // If the slashId doesn't exist, returns a request with default values
        return _slashingRequests[slashId];
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function requestSlashing(Payload calldata payload)
        external
        override
        onlyService(_msgSender())
        whenNotPaused
        returns (bytes32)
    {
        require(bytes(payload.reason).length <= 250, "reason too long");
        require(payload.mbips > 0, "mbips must be > 0");
        require(payload.timestamp <= block.timestamp, "timestamp in future");

        address service = _msgSender();
        ISLAYRegistryV2.SlashParameter memory slashParameter =
            REGISTRY.getSlashParameterAt(service, payload.operator, payload.timestamp);

        require(payload.mbips <= slashParameter.maxMbips, "mbips exceeds max allowed");
        require(
            payload.timestamp > (block.timestamp - REGISTRY.getWithdrawalDelay(payload.operator)), "timestamp too old"
        );

        bytes32 slashId = _pendingSlashingRequestIds[service][payload.operator];
        if (slashId != bytes32(0)) {
            ISLAYRouterSlashingV2.Request storage pendingRequest = _slashingRequests[slashId];

            if (pendingRequest.status == ISLAYRouterSlashingV2.Status.Locked) {
                revert("Previous slashing request lifecycle not completed");
            }

            if (pendingRequest.status == ISLAYRouterSlashingV2.Status.Pending) {
                if (pendingRequest.requestExpiry > uint32(block.timestamp)) {
                    // The previous slashing request is pending and has not expired
                    revert("Previous slashing request lifecycle not completed");
                } else {
                    // The previous slashing request is pending but expired
                    // eligible for new slashing request to take place
                    // by canceling the previous slashing request.
                    pendingRequest.status = ISLAYRouterSlashingV2.Status.Canceled;
                    emit ISLAYRouterSlashingV2.SlashingCanceled(service, payload.operator, slashId);
                }
            }
        }

        uint32 requestResolution = uint32(block.timestamp) + slashParameter.resolutionWindow;
        ISLAYRouterSlashingV2.Request memory request = ISLAYRouterSlashingV2.Request({
            status: ISLAYRouterSlashingV2.Status.Pending,
            service: service,
            mbips: payload.mbips,
            timestamp: payload.timestamp,
            requestTime: uint32(block.timestamp),
            operator: payload.operator,
            requestResolution: requestResolution,
            requestExpiry: requestResolution + SLASHING_REQUEST_EXPIRY_WINDOW
        });

        slashId = SlashingRequestId.hash(request);
        _pendingSlashingRequestIds[service][payload.operator] = slashId;
        _slashingRequests[slashId] = request;
        emit ISLAYRouterSlashingV2.SlashingRequested(service, payload.operator, slashId, request, payload.reason);
        return slashId;
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function lockSlashing(bytes32 slashId) external override whenNotPaused onlyService(_msgSender()) nonReentrant {
        ISLAYRouterSlashingV2.Request storage request = _slashingRequests[slashId];
        // Only service that initiated the slash request can call this function.
        if (request.service != _msgSender()) {
            revert ISLAYRouterSlashingV2.Unauthorized();
        }

        // Check if the slashing request is pending.
        if (request.status != ISLAYRouterSlashingV2.Status.Pending) {
            revert ISLAYRouterSlashingV2.InvalidStatus();
        }

        // Check if the slashing request has not expired
        if (request.requestExpiry < uint32(block.timestamp)) {
            revert ISLAYRouterSlashingV2.SlashingRequestExpired();
        }

        // Check if the slashing request is after the resolution window has passed
        if (request.requestResolution > uint32(block.timestamp)) {
            revert ISLAYRouterSlashingV2.SlashingResolutionNotReached();
        }

        // Update the slashing request status to locked state first
        request.status = ISLAYRouterSlashingV2.Status.Locked;

        // Iterate through the vaults and call lockSlashing on each of them
        EnumerableSet.AddressSet storage operatorVaults = _operatorVaults[request.operator];
        uint256 vaultsCount = operatorVaults.length();
        for (uint256 i = 0; i < vaultsCount;) {
            address vaultAddress = operatorVaults.at(i);
            ISLAYVaultV2 vault = ISLAYVaultV2(vaultAddress);

            // calculate the slash amount from mbips
            uint256 slashAmount = Math.mulDiv(vault.totalAssets(), request.mbips, 10_000_000);
            if (slashAmount != 0) {
                // Call the lockSlashing function on the vault
                vault.lockSlashing(slashAmount);

                // Store the locked assets in the router for further processing
                _lockedAssets[slashId].push(
                    ISLAYRouterSlashingV2.LockedAssets({amount: slashAmount, vault: vaultAddress})
                );
            }

            // vaultsCount is bounded to _maxVaultsPerOperator
            unchecked {
                i++;
            }
        }

        emit ISLAYRouterSlashingV2.SlashingLocked(request.service, request.operator, slashId);
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function getLockedAssets(bytes32 slashId)
        external
        view
        override
        returns (ISLAYRouterSlashingV2.LockedAssets[] memory)
    {
        return _lockedAssets[slashId];
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function finalizeSlashing(bytes32 slashId) external override whenNotPaused onlyService(_msgSender()) nonReentrant {
        ISLAYRouterSlashingV2.Request storage request = _slashingRequests[slashId];
        // Only service that initiated the slash request can call this function.
        if (request.service != _msgSender()) {
            revert ISLAYRouterSlashingV2.Unauthorized();
        }

        // Check if the slashing request is locked.
        if (request.status != ISLAYRouterSlashingV2.Status.Locked) {
            revert ISLAYRouterSlashingV2.InvalidStatus();
        }

        // Check guardrail approval. 0 - unset, 1 - approve, 2 - reject.
        if (_guardrailApproval[slashId] != 1) {
            revert ISLAYRouterSlashingV2.GuardrailHaveNotApproved();
        }

        // Update slash request to the finalized state first
        request.status = ISLAYRouterSlashingV2.Status.Finalized;

        // get slash parameters
        ISLAYRegistryV2.SlashParameter memory slashParameter =
            REGISTRY.getSlashParameterAt(request.service, request.operator, request.timestamp);

        // move locked assets to the slashing param destination
        ISLAYRouterSlashingV2.LockedAssets[] storage lockedAssets = _lockedAssets[slashId];
        for (uint256 i = 0; i < lockedAssets.length;) {
            ISLAYRouterSlashingV2.LockedAssets storage lockedAsset = lockedAssets[i];
            ISLAYVaultV2 vault = ISLAYVaultV2(lockedAsset.vault);
            uint256 amount = lockedAsset.amount;
            delete lockedAssets[i];

            // Transfer the locked assets to the slashing destination
            SafeERC20.safeTransfer(IERC20(vault.asset()), slashParameter.destination, amount);

            // vaultsCount is bounded to _maxVaultsPerOperator
            unchecked {
                i++;
            }
        }
        // remove the locked assets from the router
        delete _lockedAssets[slashId];

        // remove pending slashing request id for the service and operator pair
        delete _pendingSlashingRequestIds[request.service][request.operator];

        emit ISLAYRouterSlashingV2.SlashingFinalized(
            request.service, request.operator, slashId, slashParameter.destination
        );
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function cancelSlashing(bytes32 slashId) external override whenNotPaused onlyService(_msgSender()) {
        address service = _msgSender();

        ISLAYRouterSlashingV2.Request storage pendingSlashingRequest = _slashingRequests[slashId];
        if (pendingSlashingRequest.service != service) revert ISLAYRouterSlashingV2.Unauthorized();

        if (pendingSlashingRequest.status != ISLAYRouterSlashingV2.Status.Pending) {
            revert ISLAYRouterSlashingV2.InvalidStatus();
        }

        pendingSlashingRequest.status = ISLAYRouterSlashingV2.Status.Canceled;
        delete _pendingSlashingRequestIds[service][pendingSlashingRequest.operator];
        emit ISLAYRouterSlashingV2.SlashingCanceled(service, pendingSlashingRequest.operator, slashId);
    }

    /// @inheritdoc ISLAYRouterSlashingV2
    function guardrailApprove(bytes32 slashId, bool approve) external override whenNotPaused {
        // Only guardrail can call this function
        if (_guardrail == address(0)) {
            revert ISLAYRouterSlashingV2.Unauthorized();
        }
        if (_msgSender() != _guardrail) {
            revert ISLAYRouterSlashingV2.Unauthorized();
        }

        // check if the slashing request exists.
        // not checking status here as it will already be checked in finalizeSlashing.
        ISLAYRouterSlashingV2.Request storage request = _slashingRequests[slashId];
        if (request.service == address(0)) {
            revert ISLAYRouterSlashingV2.SlashingRequestNotFound();
        }

        // check if the slashing id has already been approved on by guardrail.
        if (_guardrailApproval[slashId] != 0) {
            revert ISLAYRouterSlashingV2.GuardrailAlreadyApproved();
        }

        // Guardrail approval are true - approve, false - reject.
        _guardrailApproval[slashId] = approve ? 1 : 2;

        emit ISLAYRouterSlashingV2.GuardrailApproval(slashId, approve);
    }
}
