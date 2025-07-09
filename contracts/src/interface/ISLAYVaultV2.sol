// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

import {IERC7540Redeem, IERC7540Operator} from "./IERC7540.sol";

/**
 * @dev Interface for the SLAYVault contract.
 */
interface ISLAYVaultV2 is IERC20Metadata, IERC4626, IERC7540Redeem, IERC7540Operator {
    /*//////////////////////////////////////////////////////////////
                                ERRORS
    //////////////////////////////////////////////////////////////*/

    /// @notice The operation failed because the contract is paused.
    error EnforcedPause();

    /// @notice The operation failed because the contract is not whitelisted.
    error ExpectedWhitelisted();

    /// @notice Thrown when the amount is zero.
    error ZeroAmount();

    /// @notice Must withdraw all assets
    error MustClaimAll();

    /// @notice Thrown when assets to withdraw exceed the maximum redeemable amount.
    error ExceededMaxRedeemable();

    /// @notice Thrown when the withdrawal delay has not passed.
    error WithdrawalDelayHasNotPassed();

    /// @notice Thrown when the caller is not the controller or an approved operator.
    error NotControllerOrOperator();

    /// @notice Preview functions are not supported for async flows.
    error PreviewNotSupported();

    /// @notice Thrown when a withdraw request is not found.
    error WithdrawRequestNotFound();

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
     * @notice Returns the total amount of shares pending redemption across all controllers.
     * This is the sum of all shares in pending and claimable redemption requests.
     *
     * @return The total amount of shares pending redemption.
     */
    function getTotalPendingRedemption() external view returns (uint256);
}
