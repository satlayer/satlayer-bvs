// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

/**
 * @title Initial Implementation Contract
 * @dev Used as a base for all SLAY contracts.
 * Designed to be initialized once to get an immutable address for each subsequent contract.
 * The immutable address (Proxies with empty implementation) is used to then set up the rest of the SLAY contracts
 * with immutable addresses. Allowing for cyclic-dependent contracts to be deployed with immutable references.
 */
contract InitialImpl is Initializable, UUPSUpgradeable, OwnableUpgradeable {
    /**
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor() {
        _disableInitializers();
    }

    function initialize(address initialOwner) public initializer {
        __Ownable_init(initialOwner);
        __UUPSUpgradeable_init();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
