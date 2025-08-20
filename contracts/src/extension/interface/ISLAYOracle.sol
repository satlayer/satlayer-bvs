// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

interface ISLAYOracle {
    error PriceIdNotSet(address asset);

    /**
     * @notice Returns the price ID for a given asset.
     * @param asset The ERC20 token address (vault asset) to query.
     * @return priceId The price feed identifier for the asset.
     */
    function getPriceId(address asset) external view returns (bytes32 priceId);

    /**
     * @notice Sets or updates the price ID for an asset.
     * @dev Only callable by the owner.
     * @param asset The ERC20 token address to configure.
     * @param priceId The price feed identifier to associate with the asset.
     */
    function setPriceId(address asset, bytes32 priceId) external;

    /**
     * @notice Fetches the price for a given price ID.
     * @param priceId The price feed identifier to query.
     * @return price The price in minor units (18 decimals).
     */
    function getPrice(bytes32 priceId) external view returns (uint256 price);

    /**
     * @notice Fetches the price for a given asset
     * @param asset The asset address to query.
     * @return price The price in minor units (18 decimals).
     */
    function getPrice(address asset) external view returns (uint256 price);
}
