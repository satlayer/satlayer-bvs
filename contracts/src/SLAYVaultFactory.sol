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
import {SLAYVault} from "./SLAYVault.sol";
import {SLAYRegistry} from "./SLAYRegistry.sol";
import {ISLAYVaultFactory} from "./interface/ISLAYVaultFactory.sol";

contract SLAYVaultFactory is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable,
    ISLAYVaultFactory
{
    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    address public immutable beacon;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRegistry public immutable registry;

    /// @dev Throws if called by any account other than the operator.
    modifier onlyOperator() {
        _checkOperator(_msgSender());
        _;
    }

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor(address beacon_, SLAYRegistry registry_) {
        beacon = beacon_;
        registry = registry_;
        _disableInitializers();
    }

    function initialize(address initialOwner) public initializer {
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
        __Pausable_init();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    /// @dev Throws if the sender is not the operator.
    function _checkOperator(address account) internal view virtual {
        if (!registry.isOperator(account)) {
            revert NotOperator(account);
        }
    }

    /// @inheritdoc ISLAYVaultFactory
    function create(IERC20Metadata asset) external whenNotPaused onlyOperator returns (SLAYVault) {
        address operator = _msgSender();
        string memory name = string(abi.encodePacked("SatLayer ", asset.name()));
        string memory symbol = string(abi.encodePacked("sat", asset.symbol()));

        bytes memory data = abi.encodeCall(SLAYVault.initialize, (asset, operator, name, symbol));
        BeaconProxy proxy = new BeaconProxy(beacon, data);
        return SLAYVault(address(proxy));
    }

    /// @inheritdoc ISLAYVaultFactory
    function create(IERC20 asset, address operator, string memory name, string memory symbol)
        external
        whenNotPaused
        onlyOwner
        returns (SLAYVault)
    {
        _checkOperator(operator);
        bytes memory data = abi.encodeCall(SLAYVault.initialize, (asset, operator, name, symbol));
        BeaconProxy proxy = new BeaconProxy(beacon, data);
        return SLAYVault(address(proxy));
    }
}
