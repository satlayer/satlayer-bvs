// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UpgradeableBeacon} from "@openzeppelin/contracts/proxy/beacon/UpgradeableBeacon.sol";
import {BeaconProxy} from "@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SLAYVault} from "./SLAYVault.sol";

contract SLAYVaultFactory is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    address public immutable beacon;

    /**
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor(address beacon_) {
        beacon = beacon_;
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
        _checkOperator();
        _;
    }

    /**
     * @dev Throws if the sender is not the operator.
     */
    function _checkOperator() internal view virtual {
        // TODO: we need to check the caller is an operator here.
    }

    function create(IERC20 asset_, string memory name_, string memory symbol_)
        public
        whenNotPaused
        onlyOperator
        returns (address)
    {
        // TODO: the name and symbol of the asset should be inferred and prefixed.
        bytes memory data = abi.encodeCall(SLAYVault.initialize, (asset_, name_, symbol_));
        BeaconProxy proxy = new BeaconProxy(beacon, data);
        // TODO: add to vault router.
        return address(proxy);
    }
}
