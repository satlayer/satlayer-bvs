// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {IPyth} from "@pythnetwork/pyth-sdk-solidity/IPyth.sol";
import {PythUtils} from "@pythnetwork/pyth-sdk-solidity/PythUtils.sol";
import {PythStructs} from "@pythnetwork/pyth-sdk-solidity/PythStructs.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

import {ISLAYOracle} from "./interface/ISLAYOracle.sol";
import {SLAYBase} from "../SLAYBase.sol";
import {ISLAYVaultV2} from "../interface/ISLAYVaultV2.sol";
import {ISLAYRouterV2} from "../interface/ISLAYRouterV2.sol";

/**
 * @title SLAYOracle
 * @notice SLAYOracle is an upgradeable contract that provides price feeds and AUM calculations for SLAYVaults using Pyth Network.
 * It allows setting and retrieving Pyth price IDs for an asset, fetching current prices, and calculating the total assets under management (AUM)
 * for operators based on their associated vaults.
 */
contract SLAYOracle is SLAYBase, ISLAYOracle {
    ISLAYRouterV2 internal _slayRouter;

    IPyth internal _pyth;

    uint256 public constant MAX_PRICE_AGE = 15 minutes;

    /// @dev stores the mapping of assets to their corresponding Pyth price IDs
    mapping(address asset => bytes32 priceId) internal _assetToPriceId;

    constructor(address pyth_, address slayRouter_) {
        _pyth = IPyth(pyth_);
        _slayRouter = ISLAYRouterV2(slayRouter_);

        _disableInitializers();
    }

    /// @inheritdoc ISLAYOracle
    function getPriceId(address asset) external view override returns (bytes32) {
        return _assetToPriceId[asset];
    }

    /// @inheritdoc ISLAYOracle
    function setPriceId(address asset, bytes32 priceId) external override onlyOwner {
        _assetToPriceId[asset] = priceId;
        emit PriceIdSet(asset, priceId);
    }

    /// @inheritdoc ISLAYOracle
    function getPrice(bytes32 priceId) public view override returns (uint256) {
        PythStructs.Price memory price = _pyth.getPriceNoOlderThan(priceId, MAX_PRICE_AGE);

        // convert price to uint256 and minor units (18 decimals)
        uint256 basePrice = PythUtils.convertToUint(price.price, price.expo, 18);

        return basePrice;
    }

    /// @inheritdoc ISLAYOracle
    function getPrice(address asset) public view override returns (uint256) {
        bytes32 priceId = _assetToPriceId[asset];
        if (priceId == bytes32(0)) {
            revert PriceIdNotSet(asset);
        }
        return getPrice(priceId);
    }

    /// @inheritdoc ISLAYOracle
    function getOperatorAUM(address operator) external view virtual override returns (uint256 aum) {
        address[] memory vaults = _slayRouter.getOperatorVaults(operator);
        // get each vault's total active stake
        for (uint256 i = 0; i < vaults.length; i++) {
            aum += getVaultAUM(vaults[i]);
        }
        return aum;
    }

    /// @inheritdoc ISLAYOracle
    function getVaultAUM(address vault) public view virtual returns (uint256) {
        require(vault != address(0), "Invalid vault address");

        ISLAYVaultV2 vaultI = ISLAYVaultV2(vault);

        // get the vault's total assets
        uint256 vaultTotalAssets = vaultI.totalAssets();
        if (vaultTotalAssets == 0) {
            // return early if no assets
            return 0;
        }

        // get conversion rate
        uint256 USDPricePerAsset = getPrice(vaultI.asset());

        // convert asset to USD in 18 decimals
        return Math.mulDiv(vaultTotalAssets, USDPricePerAsset, 10 ** vaultI.decimals());
    }
}
