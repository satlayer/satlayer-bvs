// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IExternalVaultConnector {
    function asset() external view returns (address); // ERC-4626 underlying
    function depositFor(address user, uint256 assets) external returns (uint256 sharesOut);
    function redeemFor(address user, uint256 requestedAssets, uint256 minAssetsOut)
        external
        returns (uint256 assetsOut, uint256 sharesBurned);
    function assetsOf(address user) external view returns (uint256); // entitlement incl. yield
}
