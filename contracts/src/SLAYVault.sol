// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {ERC165Upgradeable} from "@openzeppelin/contracts-upgradeable/utils/introspection/ERC165Upgradeable.sol";
import {ERC20PermitUpgradeable} from
    "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20PermitUpgradeable.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {ERC4626Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC4626Upgradeable.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {SLAYRegistry} from "./SLAYRegistry.sol";
import {SLAYRouter} from "./SLAYRouter.sol";
import {IERC7540Redeem, IERC7540Operator} from "./interface/IERC7540.sol";

/**
 * @dev Interface for the SLAYVault contract.
 */
interface ISLAYVault is IERC20Metadata, IERC4626, IERC7540Redeem, IERC7540Operator {}

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
    ISLAYVault
{
    using SafeERC20 for IERC20;

    /// @notice Struct representing a redeem request.
    struct RedeemRequestStruct {
        /// @notice The total amount of shares requested for redemption.
        uint256 shares;
        /// @notice The timestamp when the shares can be claimed.
        uint256 claimableAt;
    }

    /*//////////////////////////////////////////////////////////////
                            STATE VARIABLES
    //////////////////////////////////////////////////////////////*/

    /// @dev Assume requests are non-fungible and all have ID = 0
    uint256 internal constant REQUEST_ID = 0;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRouter public immutable router;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRegistry public immutable registry;

    /**
     * @notice The `delegated` is the address where the vault is delegated to.
     * This address cannot withdraw assets from the vault.
     * See https://build.satlayer.xyz/getting-started/operators for more information.
     * @dev This address is the address of the SatLayer operator that is delegated to manage the vault.
     */
    address public delegated;

    /// @notice Operator approval status per controller.
    mapping(address controller => mapping(address operator => bool)) internal _isOperator;

    /// @notice Stores all pending redemption requests for each controller.
    mapping(address controller => RedeemRequestStruct) internal _pendingRedemption;

    /// @notice Stores the total amount of shares pending redemption.
    uint256 internal _totalPendingRedemption;

    /*//////////////////////////////////////////////////////////////
                                ERRORS
    //////////////////////////////////////////////////////////////*/

    /// @notice The operation failed because the contract is paused.
    error EnforcedPause();

    /// @notice The operation failed because the contract is not whitelisted.
    error ExpectedWhitelisted();

    /// @notice Thrown when the amount is zero.
    error ZeroAmount();

    /// @notice Must withdraw all assets
    error MustClaimAll();

    /// @notice Thrown when assets to withdraw exceed the maximum redeemable amount.
    error ExceededMaxRedeemable();

    /// @notice Thrown when the withdrawal delay has not passed.
    error WithdrawalDelayHasNotPassed();

    /// @notice Thrown when the caller is not the controller or an approved operator.
    error NotControllerOrOperator();

    /// @notice Preview functions are not supported for async flows.
    error PreviewNotSupported();

    /// @notice Thrown when a withdraw request is not found.
    error WithdrawRequestNotFound();

    modifier onlyControllerOrOperator(address controller) {
        if (_msgSender() != controller && !_isOperator[controller][_msgSender()]) {
            revert NotControllerOrOperator();
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
        delegated = delegated_;
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

    /*//////////////////////////////////////////////////////////////
                              ERC7540 LOGIC
    //////////////////////////////////////////////////////////////*/

    /// @inheritdoc IERC7540Redeem
    function requestRedeem(uint256 shares, address controller, address owner) public returns (uint256 requestId) {
        // Checks
        if (shares == 0) {
            revert ZeroAmount();
        }

        // spend allowance if caller is not the owner AND not an operator
        if (owner != _msgSender() && !_isOperator[owner][_msgSender()]) {
            _spendAllowance(owner, _msgSender(), shares);
        }

        RedeemRequestStruct storage pendingRedemptionRequest = _pendingRedemption[controller];

        // increment the shares in the pending redemption request
        pendingRedemptionRequest.shares += shares;

        // reset the claimableAt to the current time + withdrawalDelay
        uint32 withdrawalDelay = registry.getWithdrawalDelay(delegated);
        pendingRedemptionRequest.claimableAt = block.timestamp + withdrawalDelay;

        // update _totalPendingRedemption
        _totalPendingRedemption += shares;

        // transfer shares from owner to the contract
        _transfer(owner, address(this), shares);
        emit RedeemRequest(controller, owner, REQUEST_ID, _msgSender(), shares);
        return REQUEST_ID;
    }

    /// @inheritdoc IERC7540Redeem
    function pendingRedeemRequest(uint256, address controller) public view returns (uint256 pendingShares) {
        RedeemRequestStruct memory request = _pendingRedemption[controller];
        if (request.claimableAt > block.timestamp) {
            return request.shares;
        }
        return 0;
    }

    /// @inheritdoc IERC7540Redeem
    function claimableRedeemRequest(uint256, address controller) public view returns (uint256 claimableShares) {
        RedeemRequestStruct memory request = _pendingRedemption[controller];
        if (request.claimableAt <= block.timestamp && request.shares > 0) {
            return request.shares;
        }
        return 0;
    }

    /// @inheritdoc IERC7540Operator
    function setOperator(address operator, bool approved) external returns (bool success) {
        _isOperator[_msgSender()][operator] = approved;
        emit OperatorSet(_msgSender(), operator, approved);
        return true;
    }

    /// @inheritdoc IERC7540Operator
    function isOperator(address controller, address operator) external view returns (bool status) {
        return _isOperator[controller][operator];
    }

    /*//////////////////////////////////////////////////////////////
                         ERC4626 OVERRIDE LOGIC
    //////////////////////////////////////////////////////////////*/

    /**
     * @inheritdoc IERC4626
     * @dev For ERC7540, the withdraw functions are used to claim the assets.
     * This function does not transfer the shares to the contract, because this already happened on requestRedeem.
     * Controller MUST be the msg.sender unless the controller has approved msg.sender as an operator.
     */
    function withdraw(uint256 assets, address receiver, address controller)
        public
        virtual
        override(IERC4626, ERC4626Upgradeable)
        onlyControllerOrOperator(controller)
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
        _withdraw(_msgSender(), receiver, controller, maxAssets, shares);
    }

    /**
     * @inheritdoc IERC4626
     * @dev For ERC7540, the redeem functions are used to claim the shares.
     * This function does not transfer the shares to the contract, because this already happened on requestRedeem.
     * Controller MUST be the msg.sender unless the controller has approved msg.sender as an operator.
     */
    function redeem(uint256 shares, address receiver, address controller)
        public
        virtual
        override(IERC4626, ERC4626Upgradeable)
        onlyControllerOrOperator(controller)
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
        _withdraw(_msgSender(), receiver, controller, assets, shares);
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

        emit Withdraw(_msgSender(), receiver, controller, assets, shares);
    }

    /// @inheritdoc IERC4626
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

    /// @inheritdoc IERC4626
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

    /// @dev For ERC7540, preview functions MUST revert for all callers and inputs.
    function previewWithdraw(uint256) public pure virtual override(IERC4626, ERC4626Upgradeable) returns (uint256) {
        revert PreviewNotSupported();
    }

    /// @dev For ERC7540, preview functions MUST revert for all callers and inputs.
    function previewRedeem(uint256) public pure virtual override(IERC4626, ERC4626Upgradeable) returns (uint256) {
        revert PreviewNotSupported();
    }
}
