// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {SLAYRegistryV2} from "../SLAYRegistryV2.sol";

/**
 * @title Vaults Router Interface
 * @dev Interface for the SLAYRouterV2 contract.
 */
interface ISLAYRouterV2 {
    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Emitted when the vault whitelist status is updated.
     */
    event VaultWhitelisted(address indexed operator, address vault, bool whitelisted);

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev Set the individual whitelist status for a SLAYVault.
     * This allows CA owner to control which vaults can be interacted with through the router.
     * For non-granular state/modifier, use {SLAYRouterV2-pause} to pause all vaults.
     * When a vault is whitelisted, it can be interacted with through the router.
     *
     * @param vault_ address of the vault to set the whitelist status for.
     * This should be a SLAYVault contract address but isn't "checked" to allow for flexible un-whitelisting of vaults.
     * @param isWhitelisted The whitelist status to set.
     */
    function setVaultWhitelist(address vault_, bool isWhitelisted) external;
}
