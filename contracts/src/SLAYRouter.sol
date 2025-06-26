// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

import {SLAYRegistry} from "./SLAYRegistry.sol";
import {ISLAYRouter} from "./interface/ISLAYRouter.sol";

/**
 * @title SLAYRouter
 * @dev The central point for managing interactions with SLAYVaults.
 * This contract is designed to work with the SLAYRegistry for managing vaults and their states.
 *
 * @custom:oz-upgrades-from src/InitialImpl.sol:InitialImpl
 */
contract SLAYRouter is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable, ISLAYRouter {
    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRegistry public immutable registry;

    mapping(address => bool) public whitelisted;

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

    /**
     * @dev Authorizes an upgrade to a new implementation.
     * This function is required by UUPS and restricts upgradeability to the contract owner.
     * @param newImplementation The address of the new contract implementation.
     */
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

    /// @inheritdoc ISLAYRouter
    function setVaultWhitelist(address vault_, bool isWhitelisted) external onlyOwner {
        whitelisted[vault_] = isWhitelisted;
        emit VaultWhitelisted(vault_, isWhitelisted);
    }
}
