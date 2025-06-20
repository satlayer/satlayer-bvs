// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./interface/IERC7540.sol";
import {ERC165Upgradeable} from "@openzeppelin/contracts-upgradeable/utils/introspection/ERC165Upgradeable.sol";
import {ERC20PermitUpgradeable} from
    "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20PermitUpgradeable.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {ERC4626Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC4626Upgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {SLAYRegistry} from "./SLAYRegistry.sol";
import {SLAYRouter} from "./SLAYRouter.sol";

/**
 * @dev Interface for the SLAYVault contract.
 */
interface ISLAYVault is IERC20Metadata, IERC4626 {}

/**
 * Implementation contract for SLAYVault.
 * This contract is not initialized directly, but through the SLAYVaultFactory using the Beacon proxy pattern.
 */
contract SLAYVault is
    Initializable,
    ERC20Upgradeable,
    ERC4626Upgradeable,
    ERC165Upgradeable,
    ERC20PermitUpgradeable,
    OwnableUpgradeable,
    ISLAYVault,
    IERC7540Redeem
{
    using SafeERC20 for IERC20;

    /**
     * @notice Struct representing a redeem request.
     */
    struct RedeemRequestStruct {
        /**
         * total amount of shares requested for redemption.
         */
        uint256 shares;
        /**
         * Timestamp when the shares can be redeemed.
         */
        uint256 claimableAt;
    }

    /* //////////////    STATE VARIABLES    ////////////// */

    /**
     * @dev Assume requests are non-fungible and all have ID = 0
     */
    uint256 internal constant REQUEST_ID = 0;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRouter public immutable router;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRegistry public immutable registry;

    /**
     * @notice The `operator` is the address where the vault is delegated to.
     * This address cannot withdraw assets from the vault.
     * See https://build.satlayer.xyz/getting-started/operators for more information.
     */
    address public operator;

    /**
     * @notice The delay in seconds before a withdrawal can be processed.
     */
    uint256 public withdrawalDelay;

    /**
     * @notice Proxy approval status per controller.
     * @dev erc7540 operator has been renamed to proxy to prevent confusion with SatLayer's Operator.
     */
    mapping(address controller => mapping(address proxy => bool)) public isProxy;

    /**
     * @notice Stores all pending redemption requests for each controller.
     */
    mapping(address controller => RedeemRequestStruct) internal _pendingRedemption;

    /**
     * @notice Stores the total amount of shares pending redemption.
     */
    uint256 internal _totalPendingRedemption;

    /* //////////////    ERRORS    ////////////// */

    /**
     * @dev The operation failed because the contract is paused.
     */
    error EnforcedPause();

    /**
     * @dev The operation failed because the contract is not whitelisted.
     */
    error ExpectedWhitelisted();

    /**
     * @notice Thrown when the amount is zero.
     */
    error ZeroAmount();

    /**
     * @notice Must withdraw all assets
     */
    error MustClaimAll();

    /**
     * @notice Thrown when assets to withdraw exceed the maximum redeemable amount.
     */
    error ExceededMaxRedeemable();

    /**
     * @notice Thrown when the withdrawal delay has not passed.
     */
    error WithdrawalDelayHasNotPassed();

    /**
     * @notice Thrown when the caller is not the controller or an approved proxy.
     */
    error NotSelfOrProxy();

    /**
     * @notice Preview functions are not supported for async flows.
     */
    error PreviewNotSupported();

    /**
     * @notice Thrown when a withdraw request is not found.
     */
    error WithdrawRequestNotFound();

    modifier onlyControllerOrProxy(address controller) {
        if (msg.sender != controller && !isProxy[controller][msg.sender]) {
            revert NotSelfOrProxy();
        }
        _;
    }

    /**
     * @dev The event emitted when a proxy is set. Renamed from `OperatorSet` to `ProxySet`.
     *
     * @param controller The address of the controller.
     * @param proxy The address of the proxy.
     * @param approved The approval status.
     */
    event ProxySet(address indexed controller, address indexed proxy, bool approved);

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor(SLAYRouter router_, SLAYRegistry registry_) {
        router = router_;
        registry = registry_;
        _disableInitializers();
    }

    /**
     * @dev Initializes the SLAYVault with the given parameters.
     * This function is called by the SLAYVaultFactory when creating a new SLAYVault instance.
     * Not to be called directly.
     */
    function initialize(IERC20 asset_, address operator_, string memory name_, string memory symbol_)
        public
        initializer
    {
        __ERC20_init(name_, symbol_);
        __ERC4626_init(asset_);
        __ERC20Permit_init(name_);
        operator = operator_;
        withdrawalDelay = withdrawalDelay_;

        // set operator as the owner of the contract
        __Ownable_init(operator_);
    }

    function decimals() public view override(ERC20Upgradeable, ERC4626Upgradeable, IERC20Metadata) returns (uint8) {
        return ERC4626Upgradeable.decimals();
    }

    /**
     * @dev See {ERC20-_update} with additional requirements for the SLAYRouter.
     *
     * To _update the balances of the SLAYVault (and therefore mint/deposit/withdraw/redeem),
     * the following conditions must be met:
     *
     * - the contract must not be paused in the SLAYRouter.
     * - the contract must be whitelisted in the SLAYRouter.
     */
    function _update(address from, address to, uint256 value) internal virtual override whenNotPaused whenWhitelisted {
        super._update(from, to, value);
    }

    /**
     * @dev Modifier to make a function callable only when the SLAYRouter is not paused.
     * SLAYVault doesn't enforce its own pause state, but relies on the SLAYRouter to manage the pause state.
     * If the SLAYRouter is paused, all operations marked with this modifier will revert with `EnforcedPause`.
     */
    modifier whenNotPaused() {
        _requireNotPaused();
        _;
    }

    /**
     * @dev Modifier to make a function callable only when the SLAYVault is whitelisted in the SLAYRouter.
     * If the SLAYVault is not whitelisted, all operations marked with this modifier will revert with `ExpectedWhitelisted`.
     */
    modifier whenWhitelisted() {
        _requireWhitelisted();
        _;
    }

    /**
     * @dev Throws if the SLAYRouter is paused.
     */
    function _requireNotPaused() internal view virtual {
        if (router.paused()) {
            revert EnforcedPause();
        }
    }

    /**
     * @dev Throws if the SLAYVault is not whitelisted in the SLAYRouter.
     */
    function _requireWhitelisted() internal view virtual {
        if (!router.whitelisted(address(this))) {
            revert ExpectedWhitelisted();
        }
    }

    /* //////////////    ERC7540 LOGIC    ////////////// */

    function requestRedeem(uint256 shares, address controller, address owner)
        public
        onlyControllerOrProxy(controller)
        returns (uint256 requestId)
    {
        // Checks
        if (shares == 0) {
            revert ZeroAmount();
        }

        // spend allowance if caller is not the owner and not a proxy
        if (owner != msg.sender) {
            if (!isProxy[owner][msg.sender]) {
                _spendAllowance(owner, msg.sender, shares);
            }
        }

        // transfer shares from owner to the contract
        SafeERC20.safeTransferFrom(this, owner, address(this), shares);

        RedeemRequestStruct memory pendingRedemptionRequest = _pendingRedemption[controller];
        pendingRedemptionRequest.shares += shares;
        // reset the claimableAt to the current time + withdrawalDelay
        pendingRedemptionRequest.claimableAt = block.timestamp + withdrawalDelay;
        // save the pending redemption request
        _pendingRedemption[controller] = pendingRedemptionRequest;

        // update _totalPendingRedemption
        _totalPendingRedemption += shares;

        emit RedeemRequest(controller, owner, REQUEST_ID, msg.sender, shares);
        return REQUEST_ID;
    }

    function pendingRedeemRequest(uint256, address controller) public view returns (uint256 pendingShares) {
        RedeemRequestStruct memory request = _pendingRedemption[controller];
        if (request.claimableAt > block.timestamp) {
            return request.shares;
        }
        return 0;
    }

    function claimableRedeemRequest(uint256, address controller) public view returns (uint256 claimableShares) {
        RedeemRequestStruct memory request = _pendingRedemption[controller];
        if (request.claimableAt <= block.timestamp && request.shares > 0) {
            return request.shares;
        }
        return 0;
    }

    function _claimableRedeemRequest(uint256 fulfilledAssets, uint256 redeemShares)
        internal
        pure
        returns (uint256 shares)
    {
        return fulfilledAssets != 0 ? redeemShares : 0;
    }

    function setWithdrawalDelay(uint256 delay) public onlyOwner {
        withdrawalDelay = delay;
    }

    /// @dev renamed from erc7540 `setOperator` to `setProxy` to avoid confusion with SatLayer's Operator.
    function setProxy(address proxy, bool approved) public returns (bool success) {
        isProxy[msg.sender][proxy] = approved;
        emit ProxySet(msg.sender, proxy, approved);
        return true;
    }

    /* //////////////    ERC4626 OVERRIDE LOGIC    ////////////// */

    function withdraw(uint256 assets, address receiver, address controller)
        public
        virtual
        override(IERC4626, ERC4626Upgradeable)
        onlyControllerOrProxy(controller)
        returns (uint256 shares)
    {
        if (assets == 0) {
            revert ZeroAmount();
        }

        RedeemRequestStruct storage request = _pendingRedemption[controller];
        if (request.claimableAt == 0) {
            revert WithdrawRequestNotFound();
        }
        if (request.claimableAt > block.timestamp) {
            revert WithdrawalDelayHasNotPassed();
        }

        shares = request.shares;
        uint256 maxAssets = convertToAssets(shares);

        // only allow full withdrawals
        if (assets < maxAssets) {
            revert MustClaimAll();
        }
        // prevent withdrawal of more assets than requested
        if (assets > maxAssets) {
            revert ExceededMaxRedeemable();
        }

        // burn, transfer and emit Withdraw event
        _withdraw(msg.sender, receiver, controller, maxAssets, shares);
    }

    function redeem(uint256 shares, address receiver, address controller)
        public
        virtual
        override(IERC4626, ERC4626Upgradeable)
        onlyControllerOrProxy(controller)
        returns (uint256 assets)
    {
        if (shares == 0) {
            revert ZeroAmount();
        }

        RedeemRequestStruct storage request = _pendingRedemption[controller];
        if (request.claimableAt == 0) {
            revert WithdrawRequestNotFound();
        }
        if (request.claimableAt > block.timestamp) {
            revert WithdrawalDelayHasNotPassed();
        }

        // only allow full redeems
        if (shares < request.shares) {
            revert MustClaimAll();
        }
        // prevent withdrawal of more shares than requested
        if (shares > request.shares) {
            revert ExceededMaxRedeemable();
        }

        // have to calculate conversion before burning
        assets = convertToAssets(shares);

        // burn, transfer and emit Withdraw event
        _withdraw(msg.sender, receiver, controller, assets, shares);
    }

    /**
     * @dev Withdraw/redeem common workflow to
     *     - burn shares from the contract (owner has transferred shares to the contract in requestRedeem)
     *     - transfer assets to the receiver
     */
    function _withdraw(address caller, address receiver, address controller, uint256 assets, uint256 shares)
        internal
        virtual
        override(ERC4626Upgradeable)
    {
        // remove the request from pending redemption
        delete _pendingRedemption[controller];

        // update state
        _totalPendingRedemption -= shares;

        // burn shares stored in the contract
        _burn(address(this), shares);

        // transfer the assets to the receiver
        SafeERC20.safeTransfer(IERC20(asset()), receiver, assets);

        emit Withdraw(msg.sender, receiver, controller, assets, shares);
    }

    function maxWithdraw(address controller)
        public
        view
        virtual
        override(IERC4626, ERC4626Upgradeable)
        returns (uint256)
    {
        RedeemRequestStruct memory request = _pendingRedemption[controller];
        if (request.claimableAt <= block.timestamp) {
            return convertToAssets(request.shares);
        }
        return 0;
    }

    function maxRedeem(address controller)
        public
        view
        virtual
        override(IERC4626, ERC4626Upgradeable)
        returns (uint256)
    {
        RedeemRequestStruct memory request = _pendingRedemption[controller];
        if (request.claimableAt <= block.timestamp) {
            return request.shares;
        }
        return 0;
    }

    // Preview functions always revert for async flows
    function previewWithdraw(uint256) public pure virtual override(IERC4626, ERC4626Upgradeable) returns (uint256) {
        revert PreviewNotSupported();
    }

    function previewRedeem(uint256) public pure virtual override(IERC4626, ERC4626Upgradeable) returns (uint256) {
        revert PreviewNotSupported();
    }
}
