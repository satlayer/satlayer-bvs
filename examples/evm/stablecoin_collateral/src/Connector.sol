// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

contract ExternalStableVaultConnector is AccessControl, ReentrancyGuard {

    bytes32 public constant ROLE_GOV = keccak256("ROLE_GOV");
    bytes32 public constant ROLE_CG  = keccak256("ROLE_CG");


    IERC4626 public targetVault;              // external ERC-4626 vault 
    IERC20   public immutable stable;         // cached underlying stable 

 
    mapping(address => uint256) public userShares; // user -> connector-held shares attributed to user
    uint256 public totalUserShares;                // sum(userShares) = targetVault.balanceOf(address(this))

  
    event Deposited(address indexed user, uint256 assetsIn, uint256 sharesMinted);
    event Redeemed(address indexed user, uint256 assetsOut, uint256 sharesBurned);
    event TargetRotated(address indexed newVault);


    constructor(address governance, address conversionGateway, IERC4626 _target) {
        require(governance != address(0) && conversionGateway != address(0), "ZERO_ADDR");
        require(address(_target) != address(0), "VAULT_ZERO");

        targetVault = _target;
        stable = IERC20(_target.asset());

        _grantRole(ROLE_GOV, governance);
        _grantRole(ROLE_CG,  conversionGateway);
    }


    /// @notice CG deposits `assets` of the stable into the external vault on behalf of `user`.
    /// @dev CG must have transferred/approved `assets` to this connector beforehand.
    function depositFor(address user, uint256 assets)
        external
        onlyRole(ROLE_CG)
        nonReentrant
        returns (uint256 sharesOut)
    {
        require(user != address(0) && assets > 0, "BAD_ARGS");

        // Pull stable from CG -> this, approve vault, deposit (shares minted to this connector)
        require(stable.transferFrom(msg.sender, address(this), assets), "TRANSFER_IN_FAIL");
        require(stable.approve(address(targetVault), assets), "APPROVE_FAIL");
        sharesOut = targetVault.deposit(assets, address(this));

        // Attribute minted shares to the user
        userShares[user] += sharesOut;
        totalUserShares  += sharesOut;

        emit Deposited(user, assets, sharesOut);
    }

    /// @notice CG redeems `requestedAssets` (stable) for `user` and receives them.
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
        totalUserShares  -= sharesNeeded;
        sharesBurned = sharesNeeded;

        // Send stable to CG
        require(stable.transfer(msg.sender, assetsOut), "TRANSFER_OUT_FAIL");

        emit Redeemed(user, assetsOut, sharesNeeded);
    }


    /// @notice Current user entitlement in stable (principal + yield via vault exchange rate).
    function assetsOf(address user) external view returns (uint256) {
        return targetVault.convertToAssets(userShares[user]);
    }

    /// @notice Connector's pooled position in stable units (just our share balance converted).
    function totalPooledAssets() external view returns (uint256) {
        uint256 sh = targetVault.balanceOf(address(this));
        return targetVault.convertToAssets(sh);
    }

    /// @notice Raw share balance this connector holds in the external vault.
    function connectorShares() external view returns (uint256) {
        return targetVault.balanceOf(address(this));
    }

    /// @notice Returns the vault base asset (stable).
    function asset() external view returns (address) {
        return address(stable);
    }

}
