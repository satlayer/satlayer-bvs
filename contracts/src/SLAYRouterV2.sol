// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {SLAYRegistryV2} from "./SLAYRegistryV2.sol";
import {ISLAYRouterV2} from "./interface/ISLAYRouterV2.sol";
import {ISLAYVaultV2} from "./interface/ISLAYVaultV2.sol";

/**
 * @title Vaults Router Contract
 * @dev The central point for managing interactions with SLAYVaults.
 * This contract is designed to work with the SLAYRegistryV2 for managing vaults and their states.
 *
 * @custom:oz-upgrades-from src/InitialImpl.sol:InitialImpl
 */
contract SLAYRouterV2 is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable, ISLAYRouterV2 {
    using EnumerableSet for EnumerableSet.AddressSet;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRegistryV2 public immutable registry;

    mapping(address => bool) public whitelisted;

    /// @dev Stores the vaults for each operator.
    mapping(address operator => EnumerableSet.AddressSet) private _operatorVaults;

    /// @dev Return the max number of vaults allowed per operator.
    uint8 private _maxVaultsPerOperator;

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
     * @dev Initializes SLAYRouterV2 contract.
     */
    function initialize() public reinitializer(2) {
        _maxVaultsPerOperator = 10;
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
    function getMaxVaultsPerOperator() external view returns (uint8) {
        return _maxVaultsPerOperator;
    }

    /// @inheritdoc ISLAYRouterV2
    function setMaxVaultsPerOperator(uint8 count) external onlyOwner {
        require(count > _maxVaultsPerOperator, "Must be greater than current");
        _maxVaultsPerOperator = count;
    }

    /// @inheritdoc ISLAYRouterV2
    function setVaultWhitelist(address vault_, bool isWhitelisted) external onlyOwner {
        require(whitelisted[vault_] != isWhitelisted, "Vault already in desired state");

        address operator = ISLAYVaultV2(vault_).delegated();
        EnumerableSet.AddressSet storage vaults = _operatorVaults[operator];

        if (isWhitelisted) {
            if (vaults.add(vault_)) {
                require(vaults.length() <= _maxVaultsPerOperator, "Exceeds max vaults per operator");
            }
        } else {
            vaults.remove(vault_);
        }

        whitelisted[vault_] = isWhitelisted;
        emit VaultWhitelisted(operator, vault_, isWhitelisted);
    }
}
