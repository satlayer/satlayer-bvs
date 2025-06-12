// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.24;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import {SLAYRouter} from "./SLAYRouter.sol";

contract SLAYRegistry is Initializable, UUPSUpgradeable, OwnableUpgradeable, PausableUpgradeable {
    SLAYRouter public immutable router;

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor(SLAYRouter router_) {
        router = router_;
        _disableInitializers();
    }

    function initialize() public reinitializer(2) {
        __Pausable_init();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
