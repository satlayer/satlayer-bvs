// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {MockERC20} from "@satlayer/contracts/test/MockERC20.sol"; // your mintable test token
import "./MockBorrowAdapter.sol"; // path to the file above

contract BorrowAdapterTest is Test {
    address public gov    = makeAddr("gov");
    address public cg     = makeAddr("cg");     // the ROLE_CG (ConversionGateway)
    address public rando  = makeAddr("rando");

    MockERC20 public WBTC; // collateral (8 decimals)
    MockERC20 public USDC; // debt token (6 decimals)

    MockBorrowAdapter public adapter;

    function setUp() public {
        WBTC = new MockERC20("Wrapped BTC", "WBTC", 8);
        USDC = new MockERC20("USD Coin", "USDC", 6);

        adapter = new MockBorrowAdapter(gov, cg);

        vm.label(address(WBTC), "WBTC");
        vm.label(address(USDC), "USDC");
        vm.label(address(adapter), "MockBorrowAdapter");
    }

    /* -------------------- roles & pause -------------------- */

    function test_roles_are_set() public {
        bytes32 RGOV = adapter.ROLE_GOV();
        bytes32 RG = adapter.ROLE_CG();
        assertTrue(adapter.hasRole(RGOV, gov), "gov must have ROLE_GOV");
        assertTrue(adapter.hasRole(RG, cg),  "cg must have ROLE_CG");
    }

    function test_onlyCG_enforced() public {
        // supply from rando => revert
        vm.expectRevert();
        vm.prank(rando);
        adapter.supplyCollateral(address(WBTC), 1, "");

        // borrow from rando => revert
        vm.expectRevert();
        vm.prank(rando);
        adapter.borrow(address(USDC), 1, "");
    }

    function test_pause_blocks_calls() public {
        // gov pauses
        vm.prank(gov);
        adapter.setPaused(true);

        // cg can't call while paused
        vm.expectRevert();
        vm.prank(cg);
        adapter.supplyCollateral(address(WBTC), 1, "");

        // unpause
        vm.prank(gov);
        adapter.setPaused(false);

        // now ok (after approvals etc.)
    }

    /* -------------------- supply / withdraw -------------------- */

    function test_supply_and_withdraw_happy_path() public {
        uint256 amt = 10e8; // 10 WBTC

        // seed CG with collateral
        WBTC.mint(cg, amt);

        // CG approves adapter to pull
        vm.startPrank(cg);
        WBTC.approve(address(adapter), type(uint256).max);

        // supply => pulls WBTC and credits ledger
        vm.expectEmit(true, false, false, true, address(adapter));
        emit BorrowAdapterBase.Supplied(address(WBTC), amt);
        adapter.supplyCollateral(address(WBTC), amt, "");

        assertEq(WBTC.balanceOf(address(adapter)), amt, "adapter holds supplied WBTC");
        assertEq(adapter.collateralBalance(address(WBTC)), amt, "ledger updated");

        // withdraw half to CG
        uint256 half = amt / 2;
        vm.expectEmit(true, false, false, true, address(adapter));
        emit BorrowAdapterBase.Withdrawn(address(WBTC), half);
        uint256 out = adapter.withdrawCollateral(address(WBTC), half, "");
        vm.stopPrank();

        assertEq(out, half, "withdraw returns amount");
        assertEq(WBTC.balanceOf(cg), half, "caller received withdrawn WBTC");
        assertEq(adapter.collateralBalance(address(WBTC)), amt - half, "ledger decreased");
    }

    function test_withdraw_reverts_if_insufficient() public {
        // no collateral supplied -> cannot withdraw
        vm.prank(cg);
        vm.expectRevert(bytes("INSUFFICIENT_COLLAT"));
        adapter.withdrawCollateral(address(WBTC), 1, "");
    }

    /* -------------------- borrow / repay -------------------- */

    function test_borrow_and_repay_happy_path() public {
        uint256 draw = 1_000_000e6; // 1,000,000 USDC

        // borrow mints USDC to CG and increases ledger
        vm.expectEmit(true, false, false, true, address(adapter));
        emit BorrowAdapterBase.Borrowed(address(USDC), draw);
        vm.prank(cg);
        adapter.borrow(address(USDC), draw, "");

        assertEq(USDC.balanceOf(cg), draw, "caller received borrowed USDC");
        assertEq(adapter.debtBalance(address(USDC)), draw, "debt ledger increased");

        // repay half
        uint256 half = draw / 2;
        vm.startPrank(cg);
        USDC.approve(address(adapter), type(uint256).max);

        vm.expectEmit(true, false, false, true, address(adapter));
        emit BorrowAdapterBase.Repaid(address(USDC), half);
        uint256 repaid = adapter.repay(address(USDC), half, "");
        vm.stopPrank();

        assertEq(repaid, half, "repay returns actual");
        assertEq(adapter.debtBalance(address(USDC)), draw - half, "debt ledger decreased");

        // repay rest (overpay guarded inside)
        vm.startPrank(cg);
        uint256 rest = draw - half;
        USDC.approve(address(adapter), type(uint256).max);
        adapter.repay(address(USDC), rest + 123, ""); // extra ignored
        vm.stopPrank();

        assertEq(adapter.debtBalance(address(USDC)), 0, "debt cleared");
    }

    function test_repay_requires_allowance_and_balance() public {
        // If CG tries to repay without balance/approval, ERC20 will revert.
        // Mint a bit and approve too little -> reverts inside transferFrom
        USDC.mint(cg, 100);
        vm.startPrank(cg);
        adapter.borrow(address(USDC), 100, ""); // mints +100 to CG, owe +100
        // reset CG balance to zero to force transferFrom fail
        USDC.transfer(address(0xdead), USDC.balanceOf(cg));
        USDC.approve(address(adapter), 100);
        vm.expectRevert(); // transferFrom underflows
        adapter.repay(address(USDC), 100, "");
        vm.stopPrank();
    }

    /* -------------------- views -------------------- */

    function test_views_after_activity() public {
        // supply 5 WBTC
        WBTC.mint(cg, 5e8);
        vm.startPrank(cg);
        WBTC.approve(address(adapter), type(uint256).max);
        adapter.supplyCollateral(address(WBTC), 5e8, "");
        // borrow 10 USDC
        adapter.borrow(address(USDC), 10e6, "");
        vm.stopPrank();

        assertEq(adapter.collateralBalance(address(WBTC)), 5e8);
        assertEq(adapter.debtBalance(address(USDC)), 10e6);
    }
}
