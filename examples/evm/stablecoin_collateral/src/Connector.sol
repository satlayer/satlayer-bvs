// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

/**
 * @title ExternalVaultConnector
 * @notice Generic per-user accounting adapter for an external ERC-4626 vault (any asset).
 *
 * @dev
 * Purpose
 *  - Bridge between SatLayer’s ConversionGateway (CG) and an external ERC-4626 vault.
 *  - Keeps a per-user ledger of shares minted in the external vault so entitlement grows with the vault’s share price.
 *
 * Key Concepts
 *  - asset: ERC20 returned by targetVault.asset(). This contract never swaps; it only holds/forwards asset.
 *  - userShares[user]: shares of the target vault accounted to that user inside this connector.
 *  - Entitlement (in assets) = targetVault.convertToAssets(userShares[user]).
 *
 * Typical Flow
 *  1) Deposit: CG has acquired `asset` and calls `depositFor(user, assets)`.
 *     - Connector pulls `assets` from CG, approves targetVault, deposits, and attributes resulting shares to `user`.
 *  2) Redeem: CG calls `redeemFor(user, requestedAssets, minAssetsOut)`.
 *     - Connector computes required shares, clips to userShares[user], redeems, and pushes assets back to CG.
 *  3) Views: off-chain or CG can query `assetsOf(user)`, `totalPooledAssets()`, `connectorShares()`.
 *
 */
contract ExternalVaultConnector is AccessControl, ReentrancyGuard {
    bytes32 public constant ROLE_GOV = keccak256("ROLE_GOV");
    bytes32 public constant ROLE_CG = keccak256("ROLE_CG");

    IERC4626 public targetVault; // external ERC-4626 vault
    IERC20 public immutable asset; // cached underlying asset

    mapping(address => uint256) public userShares; // user -> connector-held shares attributed to user
    uint256 public totalUserShares; // sum(userShares) = targetVault.balanceOf(address(this))

    event Deposited(address indexed user, uint256 assetsIn, uint256 sharesMinted);
    event Redeemed(address indexed user, uint256 assetsOut, uint256 sharesBurned);
    event TargetRotated(address indexed newVault);

    constructor(address governance, address conversionGateway, IERC4626 _target) {
        require(governance != address(0) && conversionGateway != address(0), "ZERO_ADDR");
        require(address(_target) != address(0), "VAULT_ZERO");

        targetVault = _target;
        asset = IERC20(_target.asset());

        _grantRole(ROLE_GOV, governance);
        _grantRole(ROLE_CG, conversionGateway);
    }

    /// @notice CG deposits `assets` of the asset into the external vault on behalf of `user`.
    /// @dev CG must have transferred/approved `assets` to this connector beforehand.
    function depositFor(address user, uint256 assets)
        external
        onlyRole(ROLE_CG)
        nonReentrant
        returns (uint256 sharesOut)
    {
        require(user != address(0) && assets > 0, "BAD_ARGS");

        // Pull asset from CG -> this, approve vault, deposit (shares minted to this connector)
        require(asset.transferFrom(msg.sender, address(this), assets), "TRANSFER_IN_FAIL");
        require(asset.approve(address(targetVault), assets), "APPROVE_FAIL");
        sharesOut = targetVault.deposit(assets, address(this));

        // Attribute minted shares to the user
        userShares[user] += sharesOut;
        totalUserShares += sharesOut;

        emit Deposited(user, assets, sharesOut);
    }

    /// @notice CG redeems `requestedAssets` (asset) for `user` and receives them.
    /// @dev Clips to the user's share entitlement using current exchange rate.
    function redeemFor(address user, uint256 requestedAssets, uint256 minAssetsOut)
        external
        onlyRole(ROLE_CG)
        nonReentrant
        returns (uint256 assetsOut, uint256 sharesBurned)
    {
        require(user != address(0) && requestedAssets > 0, "BAD_ARGS");

        // Convert requested assets -> shares; clip to user's available shares
        uint256 sharesNeeded = targetVault.convertToShares(requestedAssets);
        uint256 userSh = userShares[user];
        if (sharesNeeded > userSh) sharesNeeded = userSh;
        require(sharesNeeded > 0, "NO_BALANCE");

        // Redeem by shares (more precise for fee-on-withdraw vaults)
        assetsOut = targetVault.redeem(sharesNeeded, address(this), address(this));
        require(assetsOut >= minAssetsOut, "SLIPPAGE");

        // Bookkeeping
        userShares[user] = userSh - sharesNeeded;
        totalUserShares -= sharesNeeded;
        sharesBurned = sharesNeeded;

        // Send asset to CG
        require(asset.transfer(msg.sender, assetsOut), "TRANSFER_OUT_FAIL");

        emit Redeemed(user, assetsOut, sharesNeeded);
    }

    /// @notice Current user entitlement in asset (principal + yield via vault exchange rate).
    function assetsOf(address user) external view returns (uint256) {
        return targetVault.convertToAssets(userShares[user]);
    }

    /// @notice Connector's pooled position in asset units (just our share balance converted).
    function totalPooledAssets() external view returns (uint256) {
        uint256 sh = targetVault.balanceOf(address(this));
        return targetVault.convertToAssets(sh);
    }

    /// @notice Raw share balance this connector holds in the external vault.
    function connectorShares() external view returns (uint256) {
        return targetVault.balanceOf(address(this));
    }
}
