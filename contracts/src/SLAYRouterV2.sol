// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

import {SLAYRegistryV2} from "./SLAYRegistryV2.sol";
import {ISLAYRouterV2} from "./interface/ISLAYRouterV2.sol";

/**
 * @title SLAYRouterV2
 * @dev The central point for managing interactions with SLAYVaults.
 * This contract is designed to work with the SLAYRegistryV2 for managing vaults and their states.
 *
 * @custom:oz-upgrades-from src/InitialImpl.sol:InitialImpl
 */
contract SLAYRouterV2 is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable, ISLAYRouterV2 {
    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRegistryV2 public immutable registry;

    mapping(address => bool) public whitelisted;

    /**
     * @dev Set the immutable SLAYRegistryV2 proxy address for the implementation.
     * Cyclic params in constructor are possible as an InitialImpl (empty implementation) is used for an initial deployment,
     * after which all the contracts are upgraded to their respective implementations with immutable proxy addresses.
     *
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(SLAYRegistryV2 registry_) {
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

    /// @inheritdoc ISLAYRouterV2
    function setVaultWhitelist(address vault_, bool isWhitelisted) external onlyOwner {
        whitelisted[vault_] = isWhitelisted;
        emit VaultWhitelisted(vault_, isWhitelisted);
    }
}
