// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultTest is Test, TestSuite {
    function test_whitelisted() public {
        SLAYVault vault = newVault("Bitcoin", "BTC", 8);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        assertTrue(router.whitelisted(address(vault)));
        vm.stopPrank();

        MockERC20 underlying = MockERC20(vault.asset());
        address account = address(this);
        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);
        vault.deposit(100 * 10 ** vault.decimals(), account);
        assertEq(vault.balanceOf(account), 100 * 10 ** vault.decimals());
    }

    function test_notWhitelisted() public {
        SLAYVault vault = newVault();

        MockERC20 underlying = MockERC20(vault.asset());
        address account = address(this);

        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);

        vm.expectRevert(SLAYVault.ExpectedWhitelisted.selector);
        vault.deposit(100, account);
    }

    function test_paused() public {
        SLAYVault vault = newVault();

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        router.pause();
        vm.stopPrank();

        MockERC20 underlying = MockERC20(vault.asset());
        address account = address(this);

        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);

        vm.expectRevert(SLAYVault.EnforcedPause.selector);
        vault.deposit(100, account);
    }
}
