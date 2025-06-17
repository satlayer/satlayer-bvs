// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {ERC165Upgradeable} from "@openzeppelin/contracts-upgradeable/utils/introspection/ERC165Upgradeable.sol";
import {ERC20Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import {ERC4626Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC4626Upgradeable.sol";
import {ERC20PermitUpgradeable} from
    "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20PermitUpgradeable.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

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
    ISLAYVault
{
    SLAYRouter public immutable router;
    SLAYRegistry public immutable registry;

    /**
     * @dev The operation failed because the contract is paused.
     */
    error EnforcedPause();

    /**
     * @dev The operation failed because the contract is not whitelisted.
     */
    error ExpectedWhitelisted();

    /**
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(SLAYRouter router_, SLAYRegistry registry_) {
        router = router_;
        registry = registry_;
        _disableInitializers();
    }

    function initialize(IERC20 asset_, string memory name_, string memory symbol_) public initializer {
        __ERC4626_init(asset_);
        __ERC20_init(name_, symbol_);
        __ERC20Permit_init(name_);
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
}
