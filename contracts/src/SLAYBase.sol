// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

/**
 * @title Base Empty Implementation Contract
 * @dev Serves as a placeholder implementation used to reserve immutable addresses for SLAY contracts.
 * This contract is deployed once and initialized solely to setup the initial owner and pause functionality
 * before any actual SLAY contracts are deployed.
 * The reserved address (via proxies with this empty implementation) is later used to deploy
 * actual SLAY contracts with immutable references—enabling deployment of cyclically dependent contracts.
 *
 * IMPORTANT: Only ERC7201 storage layout is used in this contract.
 * DO NOT add any state variables as this is an empty implementation.
 *
 * Extended by:
 * - SLAYRegistry
 * - SLAYRouter
 * - SLAYVaultFactory
 */
contract SLAYBase is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    /**
     * @dev Initializes the contract and sets the initial owner.
     * Called once to reserve the proxy address for future immutable contract deployment.
     * `__Pausable_init()` is also initialized here to ensure that the contract can be paused before upgrade.
     *
     * @param initialOwner The address to be set as the initial owner.
     */
    function initialize(address initialOwner) public initializer {
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
        __Pausable_init();
    }

    /**
     * @dev Authorizes an upgrade to a new implementation.
     * This function is required by UUPS and restricts upgradeability to the contract owner.
     * @param newImplementation The address of the new contract implementation.
     */
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    /**
     * @dev Pauses the contract.
     * The modifiers `whenNotPaused` and `whenPaused` are available for use in derived contracts.
     * This allows the base contract to be paused before any upgrade to the actual implementation.
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @dev Unpauses the contract.
     */
    function unpause() external onlyOwner {
        _unpause();
    }
}
