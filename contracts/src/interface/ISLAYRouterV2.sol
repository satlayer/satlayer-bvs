// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

/**
 * @title Vaults Router Interface
 * @dev Interface for the SLAYRouterV2 contract, which serves as the central point for managing
 * interactions with SLAYVaults.
 */
interface ISLAYRouterV2 {
    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Emitted when the vault whitelist status is updated.
     * @param operator The address of the operator associated with the vault.
     * @param vault The address of the vault whose whitelist status was updated.
     * @param whitelisted The new whitelist status of the vault.
     */
    event VaultWhitelisted(address indexed operator, address vault, bool whitelisted);

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev This limit prevents a single operator from controlling too many vaults.
     * @return The maximum number of vaults allowed per operator.
     */
    function getMaxVaultsPerOperator() external view returns (uint8);

    /**
     * @dev The new value must be greater than the previous value to prevent
     * existing operators from exceeding the limit. Only callable by the owner.
     * @param count The new maximum number of vaults per operator.
     */
    function setMaxVaultsPerOperator(uint8 count) external;

    /**
     * @dev This allows the contract owner to control which vaults can be interacted with through the router.
     * For non-granular state control, use the pause functionality to pause all vaults.
     * When a vault is whitelisted, it can be interacted with through the router.
     * The function will revert if the vault is already in the desired state.
     *
     * @param vault_ The address of the vault to set the whitelist status for.
     * This should be a SLAYVault contract address but isn't "strictly checked" (it is not possible to enforce this)
     * to allow for flexible un-whitelisting of vaults for emergency purposes.
     *
     * Importantly: VaultFactory is not used to create vaults using the beacon pattern.
     * Vaults are NOT automatically whitelisted when created to allow for vaults flexibility.
     * This might change in the future.
     * @param isWhitelisted The whitelist status to set (true to whitelist, false to un-whitelist).
     */
    function setVaultWhitelist(address vault_, bool isWhitelisted) external;

    /**
     * @dev Check if a vault is whitelisted.
     * @param vault_ The address of the vault to check.
     * @return A boolean indicating whether the vault is whitelisted (true) or not (false).
     */
    function isVaultWhitelisted(address vault_) external view returns (bool);

    /**
     * @dev Set the guardrail address. Only callable by the owner.
     * @param guardrail The address of the new guardrail contract.
     */
    function setGuardrail(address guardrail) external;
}
