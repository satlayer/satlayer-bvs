// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

import {SLAYRegistry} from "./SLAYRegistry.sol";

contract SLAYRouter is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    SLAYRegistry public immutable registry;

    mapping(address => bool) public whitelisted;

    /**
     * @dev Emitted when the pause is triggered by `account`.
     */
    event VaultWhitelisted(address indexed vault, bool whitelisted);

    /**
     * @dev Set the immutable SLAYRegistry proxy address for the implementation.
     * Cyclic params in constructor are possible as an InitialImpl (empty implementation) is used for an initial deployment,
     * after which all the contracts are upgraded to their respective implementations with immutable proxy addresses.
     *
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(SLAYRegistry registry_) {
        registry = registry_;
        _disableInitializers();
    }

    function initialize() public reinitializer(2) {
        __Pausable_init();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    /**
     * @dev Pauses the contract, all SLAYVaults will also be paused.
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @dev Unpauses the contract, all SLAYVaults will also be unpaused.
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    /**
     * Set a individual whitelist status for a vault.
     * This allows CA owner to control which vaults can be interacted with through the router.
     * For non-granular state/modifier, use {SLAYRouter-pause} to pause all vaults.
     *
     * @param vault_ address of the vault to set the whitelist status for.
     * This should be a SLAYVault contract address but isn't "checked" to allow for flexible un-whitelisting of vaults.
     */
    function setVaultWhitelist(address vault_, bool isWhitelisted) external onlyOwner {
        whitelisted[vault_] = isWhitelisted;
        emit VaultWhitelisted(vault_, isWhitelisted);
    }
}
