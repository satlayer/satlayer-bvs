// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {BeaconProxy} from "@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {SLAYBase} from "./SLAYBase.sol";

import {SLAYVaultV2} from "./SLAYVaultV2.sol";
import {ISLAYRegistryV2} from "./interface/ISLAYRegistryV2.sol";
import {ISLAYVaultFactoryV2} from "./interface/ISLAYVaultFactoryV2.sol";

/**
 * @title Vault Factory Contract
 * @dev Factory contract for creating SLAYVaultV2 instances.
 * This contract is responsible for deploying new vaults and managing their creation.
 * It inherits from SLAYBase which provides basic functionality like initialization,
 * upgradeability, ownership, and pause/unpause functions.
 *
 * @custom:oz-upgrades-from src/SLAYBase.sol:SLAYBase
 */
contract SLAYVaultFactoryV2 is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable,
    SLAYBase,
    ISLAYVaultFactoryV2
{
    /**
     * @dev The address of the UpgradeableBeacon that points to the SLAYVaultV2 implementation.
     * This is used when creating new vault instances via BeaconProxy.
     * @custom:oz-upgrades-unsafe-allow state-variable-immutable
     */
    address public immutable BEACON;

    /**
     * @dev Reference to the SLAYRegistryV2 contract used for operator verification.
     * This is used to check if an address is registered as an operator.
     * @custom:oz-upgrades-unsafe-allow state-variable-immutable
     */
    ISLAYRegistryV2 public immutable REGISTRY;

    /**
     * @dev Modifier that restricts function access to operators only.
     * Throws if called by any account that is not registered as an operator in the SLAYRegistry.
     * Uses the _checkOperator function to verify the caller's operator status.
     */
    modifier onlyOperator() {
        _checkOperator(_msgSender());
        _;
    }

    /**
     * @dev Constructor for SLAYVaultFactoryV2.
     * Sets up the immutable beacon and registry references and disables initializers.
     *
     * @param beacon_ The address of the UpgradeableBeacon that points to the SLAYVaultV2 implementation.
     * @param registry_ The address of the SLAYRegistryV2 contract used for operator verification.
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(address beacon_, ISLAYRegistryV2 registry_) {
        BEACON = beacon_;
        REGISTRY = registry_;
        _disableInitializers();
    }

    /**
     * @dev Checks if the given account is an operator.
     * Throws if the account is not registered as an operator in the SLAYRegistry.
     *
     * @param account The address to check if it's an operator.
     */
    function _checkOperator(address account) internal view virtual {
        if (!REGISTRY.isOperator(account)) {
            revert NotOperator(account);
        }
    }

    /// @inheritdoc ISLAYVaultFactoryV2
    function create(IERC20Metadata asset) external override whenNotPaused onlyOperator returns (SLAYVaultV2) {
        address operator = _msgSender();
        string memory name = string(abi.encodePacked("SatLayer ", asset.name()));
        string memory symbol = string(abi.encodePacked("sat", asset.symbol()));

        bytes memory data = abi.encodeCall(SLAYVaultV2.initialize, (asset, operator, name, symbol));
        BeaconProxy proxy = new BeaconProxy(BEACON, data);
        return SLAYVaultV2(address(proxy));
    }

    /// @inheritdoc ISLAYVaultFactoryV2
    function create(IERC20 asset, address operator, string memory name, string memory symbol)
        external
        override
        whenNotPaused
        onlyOwner
        returns (SLAYVaultV2)
    {
        _checkOperator(operator);
        bytes memory data = abi.encodeCall(SLAYVaultV2.initialize, (asset, operator, name, symbol));
        BeaconProxy proxy = new BeaconProxy(BEACON, data);
        return SLAYVaultV2(address(proxy));
    }
}
