// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {IPyth} from "@pythnetwork/pyth-sdk-solidity/IPyth.sol";
import {PythUtils} from "@pythnetwork/pyth-sdk-solidity/PythUtils.sol";
import {PythStructs} from "@pythnetwork/pyth-sdk-solidity/PythStructs.sol";

import {ISLAYOracle} from "./interface/ISLAYOracle.sol";
import {SLAYBase} from "../SLAYBase.sol";
import {ISLAYVaultV2} from "../interface/ISLAYVaultV2.sol";
import {ISLAYRouterV2} from "../SLAYRouterV2.sol";

/**
 * @title SLAYOracle
 * @notice SLAYOracle is an upgradeable contract that provides price feeds and AUM calculations for SLAYVaults using Pyth Network.
 * It allows setting and retrieving Pyth price IDs for vaults, fetching current prices, and calculating the total assets under management (AUM)
 * for operators based on their associated vaults.
 */
contract SLAYOracle is SLAYBase, ISLAYOracle {
    ISLAYRouterV2 internal _slayRouter;

    IPyth internal pyth;

    /// @dev stores the mapping of vault addresses to their corresponding Pyth price IDs
    mapping(address vault => bytes32 priceId) internal _vaultToPriceId;

    constructor() {
        _disableInitializers();
    }

    /**
     * @dev This fn is called during the upgrade from SLAYBase to SLAYOracle.
     * @param pyth_ The address of the Pyth contract.
     * @param slayRouter_ The address of the SLAYRouterV2 contract.
     */
    function initialize2(address pyth_, address slayRouter_) external reinitializer(2) {
        pyth = IPyth(pyth_);
        _slayRouter = ISLAYRouterV2(slayRouter_);
    }

    /// @inheritdoc ISLAYOracle
    function getPriceId(address vault) external view override returns (bytes32) {
        return _vaultToPriceId[vault];
    }

    /// @inheritdoc ISLAYOracle
    function setPriceId(address vault, bytes32 priceId) external override {
        ISLAYVaultV2 slayVault = ISLAYVaultV2(vault);
        require(_msgSender() == slayVault.delegated(), "Only vault's delegated operator can set price ID");

        _vaultToPriceId[vault] = priceId;
        emit PriceIdSet(vault, priceId);
    }

    /// @inheritdoc ISLAYOracle
    function getPrice(bytes32 priceId) public view override returns (uint256) {
        PythStructs.Price memory price = pyth.getPriceNoOlderThan(priceId, 10);

        // convert price to uint256 and minor units (18 decimals)
        uint256 basePrice = PythUtils.convertToUint(price.price, price.expo, 18);

        return basePrice;
    }

    /// @inheritdoc ISLAYOracle
    function getPrice(address vault) public view override returns (uint256) {
        return getPrice(_vaultToPriceId[vault]);
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
    function getVaultAUM(address vault_) public view virtual returns (uint256) {
        ISLAYVaultV2 vault = ISLAYVaultV2(vault_);
        // get the vault's total assets
        uint256 vaultAssets = vault.totalAssets();
        // get conversion rate
        uint256 wbtcUSDPrice = getPrice(vault_);
        // convert asset to USD
        return (vaultAssets * wbtcUSDPrice) / 1e18;
    }
}
