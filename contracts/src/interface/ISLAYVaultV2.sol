// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

import {IERC7540Redeem, IERC7540Operator} from "forge-std/interfaces/IERC7540.sol";

/**
 * @title SatLayer Vault Interface
 * @notice Interface defining the functionality of the SLAYVault contract
 * @dev This interface extends ERC20Metadata, ERC4626, and ERC7540 interfaces to provide
 * a comprehensive vault implementation with asynchronous redemption capabilities
 */
interface ISLAYVaultV2 is IERC20Metadata, IERC4626, IERC7540Operator, IERC7540Redeem {
    /*//////////////////////////////////////////////////////////////
                                ERRORS
    //////////////////////////////////////////////////////////////*/

    /// @notice The operation failed because the contract is paused
    error EnforcedPause();

    /// @notice The operation failed because the contract is not whitelisted
    error ExpectedWhitelisted();

    /// @notice The operation failed because the amount specified is zero
    error ZeroAmount();

    /// @notice The operation failed because all assets must be withdrawn at once
    error MustClaimAll();

    /// @notice The operation failed because assets to withdraw exceed the maximum redeemable amount
    error ExceededMaxRedeemable();

    /// @notice The operation failed because the withdrawal delay period has not yet passed
    error WithdrawalDelayHasNotPassed();

    /// @notice The operation failed because the caller is not the controller or an approved operator
    error NotControllerOrOperator();

    /// @notice The operation failed because preview functions are not supported for asynchronous flows
    error PreviewNotSupported();

    /// @notice The operation failed because the specified withdraw request was not found
    error WithdrawRequestNotFound();

    /// @notice The operation failed because the caller is not the router contract
    error NotRouter();

    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /**
     * @notice Emitted when assets are locked for slashing
     * @param amount The amount of assets locked for slashing
     */
    event SlashingLocked(uint256 amount);

    /// @notice Struct representing a redeem request.
    struct RedeemRequestStruct {
        /// @notice The total amount of shares requested for redemption.
        uint256 shares;
        /// @notice The timestamp when the shares can be claimed.
        uint256 claimableAt;
    }

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @notice Returns the address where the vault is delegated to
     * @dev This address is the SatLayer operator that is delegated to manage the vault
     * This address cannot withdraw assets from the vault
     * See https://build.satlayer.xyz/getting-started/operators for more information
     * @return The address of the delegated operator
     */
    function delegated() external view returns (address);

    /**
     * @notice Returns whether the vault is whitelisted on SLAYRouter
     * @dev This is used to check if the vault is allowed to interact with the SLAYRouter
     * @return True if the vault is whitelisted, false otherwise
     */
    function isWhitelisted() external view returns (bool);

    /**
     * @notice Returns the amount of tokenized shares that are actively staked in the vault.
     * This amount will exclude any tokens that are pending redeem or pending withdrawal and locked slashing.
     * @return Total tokens actively staked in the vault
     */
    function totalActiveStaked() external view returns (uint256);

    /**
     * @notice Moves assets from the vault to the router contract as part of the slashing process
     * @dev Only callable by the router contract. This function is used during the slashing process
     * to transfer assets from the vault to the router for penalty distribution
     * @param amount The amount of underlying asset to move to the router
     */
    function lockSlashing(uint256 amount) external;
}
