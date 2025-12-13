// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {ISLAYVaultV2} from "../../interface/ISLAYVaultV2.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

interface ISLAYVaultYT is ISLAYVaultV2 {
    /// @dev deposit asset and mint PT and YT token to recipient
    /// @dev this is combination of deposit and mintPYT
    function depositAndMintPYT(uint256 amount, address to) external returns (uint256);

    /// @dev mint PT and YT from receipt token
    function mintPYT(uint256 shares, address to) external returns (uint256);

    /// @dev redeem PT and YT into receipt token
    function redeemPYT(uint256 amount, address to) external returns (uint256);

    /// @dev claim interest from YT token in the form of vault receipt token
    function claimInterest(address recipient) external returns (uint256);

    /// @notice get SY amount redeemable from PYT amount
    /// @dev actual redeem will need equal amount of PYT and IYT to redeem for SY
    function getSYFromPYT(uint256 pytAmount) external view returns (uint256);

    /// @dev get current exchange rate of the vault for 1 PRECISION amount
    function getCurrentExchangeRate() external view returns (uint256);

    /// @notice get accrued interest for a user
    /// @dev this is the amount of interest that can be claimed by the user
    function getAccruedInterest(address user) external view returns (uint256);
}
