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
import {ISLAYVault} from "./interface/ISLAYVault.sol";

/**
 * @title SLAYVault
 * @notice ERC4626-compliant tokenized vault designed for asynchronous redemption workflows.
 * @dev
 * - This contract is deployed via the SLAYVaultFactory using the Beacon Proxy pattern.
 * - It integrates the ERC20, ERC4626, and ERC20Permit standards with custom logic for delayed redemptions,
 *   as defined in the ERC7540 interface.
 * - Redeem requests are initiated by transferring shares to the vault and can be claimed after a configurable delay.
 * - Preview functions are intentionally disabled to prevent misuse in async flows.
 * - Immutable dependencies (`SLAYRouter` and `SLAYRegistry`) are injected at construction for efficient immutable access.
 *
 * Key Features:
 * - Asynchronous redeem request/claim pattern using `requestRedeem`, `withdraw`, and `redeem`
 * - IERC7540Operator for request/claim with configurable controller-operator relationships
 * - Upgradeable via Beacon Proxy pattern
 * - Pausing and whitelisting enforced by SLAYRouter
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

    /**
     * @dev Only allow _msgSender() to be the controller or an approved operator of the controller to call the function.
     * @param controller The address of the controller.
     */
    modifier onlyControllerOrOperator(address controller) {
        if (_msgSender() != controller && !_isOperator[controller][_msgSender()]) {
            revert NotControllerOrOperator();
        }
        _;
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
    function initialize(IERC20 asset_, address delegated_, string memory name_, string memory symbol_)
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

    /// @dev Throws if the SLAYRouter is paused.
    function _requireNotPaused() internal view virtual {
        if (router.paused()) {
            revert EnforcedPause();
        }
    }

    /// @dev Throws if the SLAYVault is not whitelisted in the SLAYRouter.
    function _requireWhitelisted() internal view virtual {
        if (!router.whitelisted(address(this))) {
            revert ExpectedWhitelisted();
        }
    }

    /// @inheritdoc ISLAYVault
    function getTotalPendingRedemption() external view returns (uint256) {
        return _totalPendingRedemption;
    }

    /**
     * @dev Support the most common interfaces for SLAYVault. There might be more interfaces not listed here.
     *
     * @inheritdoc ERC165Upgradeable
     */
    function supportsInterface(bytes4 interfaceId) public view virtual override(ERC165Upgradeable) returns (bool) {
        return interfaceId == type(IERC20).interfaceId || interfaceId == type(IERC20Metadata).interfaceId
            || interfaceId == type(IERC4626).interfaceId || interfaceId == type(IERC7540Redeem).interfaceId
            || interfaceId == type(IERC7540Operator).interfaceId || interfaceId == type(ISLAYVault).interfaceId
            || super.supportsInterface(interfaceId);
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

        // if the controller is not the sender, check that the controller has msg.sender set as the operator
        if (controller != _msgSender() && !_isOperator[controller][_msgSender()]) {
            revert NotControllerOrOperator();
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
        RedeemRequestStruct storage request = _pendingRedemption[controller];
        if (request.claimableAt > block.timestamp) {
            return request.shares;
        }
        return 0;
    }

    /// @inheritdoc IERC7540Redeem
    function claimableRedeemRequest(uint256, address controller) public view returns (uint256 claimableShares) {
        RedeemRequestStruct storage request = _pendingRedemption[controller];
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

        RedeemRequestStruct memory request = _pendingRedemption[controller];
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

        RedeemRequestStruct memory request = _pendingRedemption[controller];
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

        emit Withdraw(caller, receiver, controller, assets, shares);
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
