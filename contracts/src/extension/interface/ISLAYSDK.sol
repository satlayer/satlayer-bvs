// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

/**
 * @title ISLAYSDK
 * @notice Interface for SatLayer SDK helper utilities.
 */
interface ISLAYSDK {
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
