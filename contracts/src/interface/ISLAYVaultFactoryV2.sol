// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SLAYVaultV2} from "../SLAYVaultV2.sol";
import {SLAYRegistryV2} from "../SLAYRegistryV2.sol";

/**
 * @title Vault Factory Interface
 * @dev Interface for the SLAYVaultFactory contract.
 */
interface ISLAYVaultFactoryV2 {
    /*//////////////////////////////////////////////////////////////
                                ERRORS
    //////////////////////////////////////////////////////////////*/

    /**
     * @dev The account is not an operator.
     */
    error NotOperator(address account);

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @notice For operator (the caller) to create a new SLAYVault instance using the Beacon proxy pattern.
     * The IERC20Metadata is used to initialize the vault with its name and symbol prefixed.
     * This self-serve function allows operators to create new vaults without needing to go through the owner.
     *
     * @param asset The ERC20Metadata asset to be used in the vault.
     * @return The newly created SLAYVault instance.
     */
    function create(IERC20Metadata asset) external returns (SLAYVaultV2);

    /**
     * @notice For owner to create a new SLAYVault instance using the Beacon proxy pattern.
     * This function allows the owner to create a vault with a custom operator, name, and symbol.
     * This scenario is mainly used for creating vaults that aren't IERC20Metadata compliant.
     *
     * @param asset The ERC20 asset to be used in the vault.
     * @param operator The address that will be the operator of the vault.
     * @param name The name of the tokenized vault token.
     * @param symbol The symbol of the tokenized vault token.
     * @return The newly created SLAYVault instance.
     */
    function create(IERC20 asset, address operator, string memory name, string memory symbol)
        external
        returns (SLAYVaultV2);
}
