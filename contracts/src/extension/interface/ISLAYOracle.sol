// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

/**
 * @title SatLayer Oracle Interface
 * @dev Interface for the SatLayer Oracle contract to manage and retrieve price data for vaults.
 */
interface ISLAYOracle {
    /**
     * @dev Emitted when a price ID is not set for an asset.
     * @param asset The asset address.
     */
    error PriceIdNotSet(address asset);

    /**
     * @notice Emitted when a price ID is set or updated for an asset.
     * @param asset The asset address.
     * @param priceId The price feed identifier associated with the asset.
     */
    event PriceIdSet(address indexed asset, bytes32 indexed priceId);

    /**
     * @notice Returns the price ID for a given asset.
     * @param asset The asset address to query.
     * @return priceId The price feed identifier for the asset.
     */
    function getPriceId(address asset) external view returns (bytes32);

    /**
     * @notice Sets or updates the price ID for an asset.
     * @dev Only callable by the contract owner.
     * @param asset The asset address as key.
     * @param priceId The price feed identifier to associate with the asset.
     */
    function setPriceId(address asset, bytes32 priceId) external;

    /**
     * @notice Fetches the price for a given price ID.
     * @param priceId The price feed identifier to query.
     * @return price The price in minor units (18 decimals).
     */
    function getPrice(bytes32 priceId) external view returns (uint256);

    /**
     * @notice Fetches the price for a given asset
     * @param asset The asset address to query.
     * @return price The price in minor units (18 decimals).
     */
    function getPrice(address asset) external view returns (uint256);

    /**
     * @notice Computes an operator's total Assets Under Management (AUM) in USD minor units (18 decimals).
     * @param operator The operator address whose vaults will be aggregated.
     * @return aum Total AUM in USD expressed with 18 decimals.
     */
    function getOperatorAUM(address operator) external view returns (uint256 aum);

    /**
     * @notice Computes a vault's Assets Under Management (AUM) in USD minor units (18 decimals).
     * @param vault The vault address to query.
     * @return aum Vault AUM in USD expressed with 18 decimals.
     */
    function getVaultAUM(address vault) external view returns (uint256);
}
