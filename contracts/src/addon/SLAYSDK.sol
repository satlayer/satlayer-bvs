// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {IPyth} from "@pythnetwork/pyth-sdk-solidity/IPyth.sol";
import {PythStructs} from "@pythnetwork/pyth-sdk-solidity/PythStructs.sol";
import {PythUtils} from "@pythnetwork/pyth-sdk-solidity/PythUtils.sol";

import {ISLAYRouterV2} from "../SLAYRouterV2.sol";
import {ISLAYVaultV2} from "../SLAYVaultV2.sol";

contract SLAYSDK {
    ISLAYRouterV2 internal _slayRouter;

    IPyth internal pyth;

    /// @dev stores the mapping of asset addresses to their corresponding Pyth price IDs
    mapping(address asset => bytes32 priceId) internal _assetToPriceId;

    constructor(address slayRouter_, address pyth_) {
        _slayRouter = ISLAYRouterV2(slayRouter_);
        pyth = IPyth(pyth_);
    }

    function getPythPriceId(address asset) external view returns (bytes32) {
        return _assetToPriceId[asset];
    }

    function setPythPriceId(address asset, bytes32 priceId) external {
        _assetToPriceId[asset] = priceId;
    }

    function getOperatorAUM(address _operator) external view virtual returns (uint256 aum) {
        address[] memory vaults = _slayRouter.getOperatorVaults(_operator);
        // get each vault's total active stake
        for (uint256 i = 0; i < vaults.length; i++) {
            aum += getVaultAUM(vaults[i]);
        }
        return aum;
    }

    function getVaultAUM(address _vault) public view virtual returns (uint256 aum) {
        ISLAYVaultV2 vault = ISLAYVaultV2(_vault);
        // get the vault's total assets
        uint256 vaultAssets = vault.totalAssets();
        // get conversion rate
        uint256 wbtcUSDPrice = _getPrice(_assetToPriceId[vault.asset()]);
        // convert asset to USD
        return (vaultAssets * wbtcUSDPrice) / 1e18;
    }

    // returns the price from Pyth in minor units (18 decimals)
    function _getPrice(bytes32 priceId) internal view returns (uint256) {
        PythStructs.Price memory price = pyth.getPriceNoOlderThan(priceId, 10);

        // convert price to uint256 and minor units (18 decimals)
        uint256 basePrice = PythUtils.convertToUint(price.price, price.expo, 18);

        return basePrice;
    }
}
