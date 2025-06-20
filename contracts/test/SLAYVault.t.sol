// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultTest is Test, TestSuite {
    MockERC20 public underlying = new MockERC20("Wrapped Bitcoin", "WBTC", 8);
    address public immutable operator = makeAddr("Operator Y");

    function setUp() public override {
        TestSuite.setUp();

        vm.startPrank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");
        vm.stopPrank();
    }

    function test_whitelisted() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        assertTrue(router.whitelisted(address(vault)));
        vm.stopPrank();

        address account = address(this);
        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);
        vault.deposit(100 * 10 ** vault.decimals(), account);
        assertEq(vault.balanceOf(account), 100 * 10 ** vault.decimals());
    }

    function test_notWhitelisted() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        address account = address(this);

        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);

        vm.expectRevert(SLAYVault.ExpectedWhitelisted.selector);
        vault.deposit(100, account);
    }

    function test_paused() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        router.pause();
        vm.stopPrank();

        address account = address(this);

        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);

        vm.expectRevert(SLAYVault.EnforcedPause.selector);
        vault.deposit(100, account);
    }
}
