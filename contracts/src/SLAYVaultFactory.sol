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

contract SLAYVaultFactory is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    address public immutable beacon;

    /// @custom:oz-upgrades-unsafe-allow state-variable-immutable
    SLAYRegistry public immutable registry;

    /**
     * @dev The account is not an operator.
     */
    error NotOperator(address account);

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

    /**
     * @dev Throws if called by any account other than the operator.
     */
    modifier onlyOperator() {
        _checkOperator(_msgSender());
        _;
    }

    /**
     * @dev Throws if the sender is not the operator.
     */
    function _checkOperator(address account) internal view virtual {
        if (!registry.isOperator(account)) {
            revert NotOperator(account);
        }
    }

    /**
     * @notice For operator (the caller) to create a new SLAYVault instance using the Beacon proxy pattern.
     * The IERC20Metadata is used to initialize the vault with its name and symbol prefixed.
     * This self-serve function allows operators to create new vaults without needing to go through the owner.
     *
     * @param asset The ERC20Metadata asset to be used in the vault.
     * @return The newly created SLAYVault instance.
     */
    function create(IERC20Metadata asset) public whenNotPaused onlyOperator returns (SLAYVault) {
        address operator = _msgSender();
        string memory name = string(abi.encodePacked("SatLayer ", asset.name()));
        string memory symbol = string(abi.encodePacked("sat", asset.symbol()));

        bytes memory data = abi.encodeCall(SLAYVault.initialize, (asset, operator, name, symbol, withdrawalDelay));
        BeaconProxy proxy = new BeaconProxy(beacon, data);
        return SLAYVault(address(proxy));
    }

    /**
     * @dev For owner to create a new SLAYVault instance using the Beacon proxy pattern.
     * This function allows the owner to create a vault with a custom operator, name, and symbol.
     * This scenario is mainly used for creating vaults that aren't IERC20Metadata compliant.
     *
     * @param asset The ERC20 asset to be used in the vault.
     * @param operator The address that will be the operator of the vault.
     * @param name The name of the tokenized vault token.
     * @param symbol The symbol of the tokenized vault token.
     * @return The newly created SLAYVault instance.
     */
    function create(IERC20 asset, address operator, string memory name, string memory symbol, uint256 withdrawalDelay)
        public
        whenNotPaused
        onlyOwner
        returns (SLAYVault)
    {
        _checkOperator(operator);
        bytes memory data = abi.encodeCall(SLAYVault.initialize, (asset, operator, name, symbol, withdrawalDelay));
        BeaconProxy proxy = new BeaconProxy(beacon, data);
        return SLAYVault(address(proxy));
    }
}
