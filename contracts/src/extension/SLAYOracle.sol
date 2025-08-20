// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {IPyth} from "@pythnetwork/pyth-sdk-solidity/IPyth.sol";
import {PythUtils} from "@pythnetwork/pyth-sdk-solidity/PythUtils.sol";
import {PythStructs} from "@pythnetwork/pyth-sdk-solidity/PythStructs.sol";

import {ISLAYOracle} from "./interface/ISLAYOracle.sol";
import {SLAYBase} from "../SLAYBase.sol";

contract SLAYOracle is SLAYBase, ISLAYOracle {
    IPyth internal pyth;

    /// @dev stores the mapping of asset addresses to their corresponding Pyth price IDs
    mapping(address asset => bytes32 priceId) internal _assetToPriceId;

    constructor() {
        _disableInitializers();
    }

    /**
     * @dev This fn is called during the upgrade from SLAYBase to SLAYOracle.
     * @param pyth_ The address of the Pyth contract.
     */
    function initialize2(address pyth_) external reinitializer(2) {
        pyth = IPyth(pyth_);
    }

    /// @inheritdoc ISLAYOracle
    function getPriceId(address asset) external view override returns (bytes32) {
        return _assetToPriceId[asset];
    }

    /// @inheritdoc ISLAYOracle
    function setPriceId(address asset, bytes32 priceId) external override onlyOwner {
        _assetToPriceId[asset] = priceId;
    }

    /// @inheritdoc ISLAYOracle
    function getPrice(bytes32 priceId) public view override returns (uint256) {
        PythStructs.Price memory price = pyth.getPriceNoOlderThan(priceId, 10);

        // convert price to uint256 and minor units (18 decimals)
        uint256 basePrice = PythUtils.convertToUint(price.price, price.expo, 18);

        return basePrice;
    }

    /// @inheritdoc ISLAYOracle
    function getPrice(address asset) external view override returns (uint256) {
        return getPrice(_assetToPriceId[asset]);
    }
}
