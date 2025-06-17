// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVault.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {Test, console} from "forge-std/Test.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultTest is Test, TestSuite {
    function test_token1() public {
        MockERC20 underlying = new MockERC20("Mock Token", "MTK", 12);
        address proxy = vaultFactory.create(underlying, "SLAY TokenName", "SLAY.MTK");
        SLAYVault vault = SLAYVault(proxy);
        assertEq(vault.decimals(), 12);
        assertEq(vault.symbol(), "SLAY.MTK");
    }

    function test_token2() public {
        MockERC20 underlying = new MockERC20("Mock Token AAA", "AAA", 15);
        address proxy = vaultFactory.create(underlying, "SLAY AAA", "SLAY.AAA");
        SLAYVault vault = SLAYVault(proxy);
        assertEq(vault.decimals(), 15);
        assertEq(vault.symbol(), "SLAY.AAA");
    }

    function test_whitelisted() public {
        SLAYVault vault = newVault("Bitcoin", "BTC", 8);

        vm.startPrank(owner);
        router.setWhitelist(address(vault), true);
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
        router.setWhitelist(address(vault), true);
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
