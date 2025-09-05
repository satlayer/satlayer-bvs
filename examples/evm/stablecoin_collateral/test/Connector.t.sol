// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC4626} from "@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

import {MockERC20} from "@satlayer/contracts/test/MockERC20.sol";

import "../src/Connector.sol";

contract Simple4626 is ERC20, ERC4626 {
    uint8 private immutable _dec;

    constructor(ERC20 underlying, string memory name_, string memory symbol_)
        ERC20(name_, symbol_)
        ERC4626(underlying)
    {
        _dec = underlying.decimals();
    }

    function decimals() public view override(ERC20, ERC4626) returns (uint8) {
        return _dec;
    }
}

contract ConnectorTest is Test {
    // roles
    address public gov = makeAddr("gov");
    address public cg = makeAddr("cg");
    address public rando = makeAddr("rando");

    // tokens
    MockERC20 public USDC; // 6 decimals

    Simple4626 public vault;
    ExternalVaultConnector public connector;

    function setUp() public {
        USDC = new MockERC20("USD Coin", "USDC", 6);
        vault = new Simple4626(ERC20(address(USDC)), "Vault Share", "vSHARE");
        connector = new ExternalVaultConnector(gov, cg, IERC4626(address(vault)));

        vm.label(address(USDC), "USDC");
        vm.label(address(vault), "OZ4626");
        vm.label(address(connector), "Connector");
    }

    function test_constructor_sets_asset_and_roles() public {
        assertEq(address(connector.asset()), address(USDC), "stable cache");
        assertEq(address(connector.targetVault()), address(vault), "target vault");
        assertTrue(connector.hasRole(connector.ROLE_GOV(), gov));
        assertTrue(connector.hasRole(connector.ROLE_CG(), cg));
    }

    function test_depositFor_happy_path() public {
        address user = makeAddr("alice");
        uint256 amt = 1_000_000_000;

        USDC.mint(cg, amt);

        vm.startPrank(cg);
        USDC.approve(address(connector), type(uint256).max);

        vm.expectEmit(true, true, false, true, address(connector));
        emit ExternalVaultConnector.Deposited(user, amt, amt);

        uint256 sharesOut = connector.depositFor(user, amt);
        vm.stopPrank();

        // First deposit: 1:1 shares
        assertEq(sharesOut, amt);
        assertEq(connector.userShares(user), amt);
        assertEq(connector.totalUserShares(), amt);

        // Connector holds the vault shares
        assertEq(ERC20(address(vault)).balanceOf(address(connector)), amt);

        // Vault holds the USDC
        assertEq(USDC.balanceOf(address(vault)), amt);
        assertEq(USDC.balanceOf(address(connector)), 0);

        // Views
        assertEq(connector.assetsOf(user), amt);
        assertEq(connector.connectorShares(), ERC20(address(vault)).balanceOf(address(connector)));
        assertEq(connector.totalPooledAssets(), USDC.balanceOf(address(vault)));
    }

    function test_depositFor_onlyCG() public {
        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, rando, connector.ROLE_CG())
        );
        vm.prank(rando);
        connector.depositFor(rando, 1);
    }

    function test_depositFor_badArgs() public {
        vm.prank(cg);
        vm.expectRevert(bytes("BAD_ARGS"));
        connector.depositFor(address(0), 1);

        vm.prank(cg);
        vm.expectRevert(bytes("BAD_ARGS"));
        connector.depositFor(makeAddr("u"), 0);
    }

    function _seed(address user, uint256 amt) internal {
        USDC.mint(cg, amt);
        vm.startPrank(cg);
        USDC.approve(address(connector), type(uint256).max);
        connector.depositFor(user, amt);
        vm.stopPrank();
    }

    function test_redeemFor_happy_with_yield_and_clip() public {
        address user = makeAddr("bob");
        uint256 first = 1_000_000_000;
        _seed(user, first);

        // Simulate yield: donate 100 USDC directly to the vault
        USDC.mint(address(vault), 100_000_000);

        // Read entitlement from the connector (may be 1 wei under )
        uint256 entitlement = connector.assetsOf(user);
        // Allow a 1-wei tolerance for Openzeppelin ERC4626 rounding
        assertApproxEqAbs(entitlement, 1_100_000_000, 1, "entitlement rounding");

        // Ask to redeem more than entitlement -> clip to entitlement
        uint256 request = entitlement + 123; // anything >= entitlement
        uint256 minOut = entitlement - 1; // low enough to pass

        // Expect the event with the runtime  entitlement
        vm.expectEmit(true, true, false, true, address(connector));
        emit ExternalVaultConnector.Redeemed(user, entitlement, first);

        vm.prank(cg);
        (uint256 assetsOut, uint256 burned) = connector.redeemFor(user, request, minOut);

        assertEq(assetsOut, entitlement, "assets out == entitlement after clipping");
        assertEq(burned, first, "burned all user shares");
        assertEq(connector.userShares(user), 0, "user shares cleared");
        assertEq(connector.totalUserShares(), 0, "global shares cleared");
        assertEq(USDC.balanceOf(cg), entitlement, "CG received funds");
    }

    function test_redeemFor_slippage_reverts() public {
        address user = makeAddr("carol");
        _seed(user, 500_000_000);

        // donate 50 USDC -> entitlement = around 550
        USDC.mint(address(vault), 50_000_000);

        uint256 entitlement = connector.assetsOf(user);
        assertApproxEqAbs(entitlement, 550_000_000, 1, "entitlement rounding");

        // minOut too high -> SLIPPAGE
        vm.prank(cg);
        vm.expectRevert(bytes("SLIPPAGE"));
        connector.redeemFor(user, entitlement, entitlement + 1);
    }

    function test_redeemFor_badArgs_and_noBalance() public {
        address user = makeAddr("dave");

        // only CG
        vm.expectRevert(
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, rando, connector.ROLE_CG())
        );
        vm.prank(rando);
        connector.redeemFor(user, 1, 0);

        // bad args
        vm.prank(cg);
        vm.expectRevert(bytes("BAD_ARGS"));
        connector.redeemFor(address(0), 1, 0);

        vm.prank(cg);
        vm.expectRevert(bytes("BAD_ARGS"));
        connector.redeemFor(user, 0, 0);

        // no balance
        vm.prank(cg);
        vm.expectRevert(bytes("NO_BALANCE"));
        connector.redeemFor(user, 1, 0);
    }

    function test_views_post_multiple_deposits_and_yield() public {
        address u1 = makeAddr("u1");
        address u2 = makeAddr("u2");

        _seed(u1, 300_000_000);
        _seed(u2, 700_000_000);

        assertEq(ERC20(address(vault)).balanceOf(address(connector)), 1_000_000_000);
        assertEq(connector.totalUserShares(), 1_000_000_000);

        // donate 100 yield
        USDC.mint(address(vault), 100_000_000);

        // entitlements reflect yield (allow 1 wei tolerance)
        assertApproxEqAbs(connector.assetsOf(u1), 330_000_000, 1, "u1 entitlement");
        assertApproxEqAbs(connector.assetsOf(u2), 770_000_000, 1, "u2 entitlement");

        // total pooled equals vault underlying balance
        assertApproxEqAbs(connector.totalPooledAssets(), USDC.balanceOf(address(vault)), 1, "total pooled rounding");

        // connectorShares view mirrors vault.share balance
        assertEq(connector.connectorShares(), ERC20(address(vault)).balanceOf(address(connector)));
    }
}
