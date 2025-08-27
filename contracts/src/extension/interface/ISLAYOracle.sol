// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

interface ISLAYOracle {
    error PriceIdNotSet(address vault);

    event PriceIdSet(address indexed vault, bytes32 indexed priceId);

    /**
     * @notice Returns the price ID for a given vault.
     * @param vault The vault address the query.
     * @return priceId The price feed identifier for the vault.
     */
    function getPriceId(address vault) external view returns (bytes32);

    /**
     * @notice Sets or updates the price ID for an vault.
     * @dev Only callable by the owner.
     * @param vault The vault address as key.
     * @param priceId The price feed identifier to associate with the vault.
     */
    function setPriceId(address vault, bytes32 priceId) external;

    /**
     * @notice Fetches the price for a given price ID.
     * @param priceId The price feed identifier to query.
     * @return price The price in minor units (18 decimals).
     */
    function getPrice(bytes32 priceId) external view returns (uint256);

    /**
     * @notice Fetches the price for a given vault
     * @param vault The vault address to query.
     * @return price The price in minor units (18 decimals).
     */
    function getPrice(address vault) external view returns (uint256);

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
