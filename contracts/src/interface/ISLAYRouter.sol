// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {SLAYRegistry} from "../SLAYRegistry.sol";

/**
 * @dev Interface for the SLAYRouter contract.
 */
interface ISLAYRouter {
    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Emitted when the vault whitelist status is updated.
     */
    event VaultWhitelisted(address indexed vault, bool whitelisted);

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * Set a individual whitelist status for a vault.
     * This allows CA owner to control which vaults can be interacted with through the router.
     * For non-granular state/modifier, use {SLAYRouter-pause} to pause all vaults.
     *
     * @param vault_ address of the vault to set the whitelist status for.
     * This should be a SLAYVault contract address but isn't "checked" to allow for flexible un-whitelisting of vaults.
     * @param isWhitelisted The whitelist status to set.
     */
    function setVaultWhitelist(address vault_, bool isWhitelisted) external;
}
