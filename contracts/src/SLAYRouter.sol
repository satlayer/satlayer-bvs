// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

import {SLAYRegistry} from "./SLAYRegistry.sol";

contract SLAYRouter is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    SLAYRegistry public immutable registry;

    /// Set the immutable SLAYRouter proxy address for the implementation.
    /// Cyclic construction are possible as an EmptyImpl is used for an initial deployment,
    /// after which all the contracts are upgraded to their respective implementations with immutable proxy addresses.
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor(SLAYRegistry registry_) {
        registry = registry_;
        _disableInitializers();
    }

    function initialize() public reinitializer(2) {
        __Pausable_init();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
