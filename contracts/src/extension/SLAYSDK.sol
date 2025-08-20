// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {ISLAYSDK} from "./interface/ISLAYSDK.sol";
import {ISLAYRouterV2} from "../SLAYRouterV2.sol";
import {ISLAYVaultV2} from "../SLAYVaultV2.sol";
import {SLAYBase} from "../SLAYBase.sol";
import {ISLAYOracle} from "./interface/ISLAYOracle.sol";

contract SLAYSDK is ISLAYSDK, SLAYBase {
    ISLAYRouterV2 internal _slayRouter;

    ISLAYOracle internal _slayOracle;

    constructor() {
        _disableInitializers();
    }

    /**
     * @dev This fn is called during the upgrade from SLAYBase to SLAYSDK.
     * @param slayRouter_ The address of the SLAYRouterV2 contract.
     */
    function initialize2(address slayRouter_, address slayOracle_) public reinitializer(2) {
        _slayRouter = ISLAYRouterV2(slayRouter_);
        _slayOracle = ISLAYOracle(slayOracle_);
    }

    /// @inheritdoc ISLAYSDK
    function getOperatorAUM(address operator) external view virtual override returns (uint256 aum) {
        address[] memory vaults = _slayRouter.getOperatorVaults(operator);
        // get each vault's total active stake
        for (uint256 i = 0; i < vaults.length; i++) {
            aum += getVaultAUM(vaults[i]);
        }
        return aum;
    }

    /// @inheritdoc ISLAYSDK
    function getVaultAUM(address vault) public view virtual returns (uint256) {
        ISLAYVaultV2 vault = ISLAYVaultV2(vault);
        // get the vault's total assets
        uint256 vaultAssets = vault.totalAssets();
        // get conversion rate
        uint256 wbtcUSDPrice = _slayOracle.getPrice(vault.asset());
        // convert asset to USD
        return (vaultAssets * wbtcUSDPrice) / 1e18;
    }
}
