// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UpgradeableBeacon} from "@openzeppelin/contracts/proxy/beacon/UpgradeableBeacon.sol";
import {BeaconProxy} from "@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SLAYVaultV2} from "./SLAYVaultV2.sol";
import {ISLAYRegistryV2} from "./interface/ISLAYRegistryV2.sol";
import {ISLAYVaultFactoryV2} from "./interface/ISLAYVaultFactoryV2.sol";

/**
 * @title Vault Factory Contract
 * @dev Factory contract for creating SLAYVaultV2 instances.
 * This contract is responsible for deploying new vaults and managing their creation.
 */
contract SLAYVaultFactoryV2 is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable,
    ISLAYVaultFactoryV2
{
    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    address public immutable beacon;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    ISLAYRegistryV2 public immutable registry;

    /// @dev Throws if called by any account other than the operator.
    modifier onlyOperator() {
        _checkOperator(_msgSender());
        _;
    }

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor(address beacon_, ISLAYRegistryV2 registry_) {
        beacon = beacon_;
        registry = registry_;
        _disableInitializers();
    }

    /**
     * @dev Initializes the contract and sets the initial owner.
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

    /// @dev Throws if the sender is not the operator.
    function _checkOperator(address account) internal view virtual {
        if (!registry.isOperator(account)) {
            revert NotOperator(account);
        }
    }

    /// @inheritdoc ISLAYVaultFactoryV2
    function create(IERC20Metadata asset) external override whenNotPaused onlyOperator returns (SLAYVaultV2) {
        address operator = _msgSender();
        string memory name = string(abi.encodePacked("SatLayer ", asset.name()));
        string memory symbol = string(abi.encodePacked("sat", asset.symbol()));

        bytes memory data = abi.encodeCall(SLAYVaultV2.initialize, (asset, operator, name, symbol));
        BeaconProxy proxy = new BeaconProxy(beacon, data);
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
        BeaconProxy proxy = new BeaconProxy(beacon, data);
        return SLAYVaultV2(address(proxy));
    }
}
