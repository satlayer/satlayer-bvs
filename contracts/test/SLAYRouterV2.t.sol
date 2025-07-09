// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYRouterV2.sol";
import "../src/SLAYVaultV2.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "./TestSuiteV2.sol";
import {ISLAYRouterV2} from "../src/interface/ISLAYRouterV2.sol";

contract SLAYRouterV2Test is Test, TestSuiteV2 {
    function test_defaults() public view {
        assertEq(router.owner(), owner);
        assertEq(router.paused(), false);
    }

    function test_paused() public {
        vm.prank(owner);
        router.pause();

        assertTrue(router.paused());
    }

    function test_pausedOnlyOwnerError() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.pause();
    }

    function test_unpausedOnlyOwnerError() public {
        vm.prank(owner);
        router.pause();

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.unpause();
    }

    function test_Whitelisted() public {
        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        assertFalse(router.isVaultWhitelisted(vault));

        vm.prank(owner);
        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(operator, vault, true);
        router.setVaultWhitelist(vault, true);

        assertTrue(router.isVaultWhitelisted(vault));

        vm.prank(owner);
        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(operator, vault, false);
        router.setVaultWhitelist(vault, false);

        assertFalse(router.isVaultWhitelisted(vault));
    }

    function test_Whitelisted_NotVault() public {
        address vault = vm.randomAddress();
        assertFalse(router.isVaultWhitelisted(vault));

        vm.prank(owner);
        vm.expectRevert();
        router.setVaultWhitelist(vault, true);

        assertFalse(router.isVaultWhitelisted(vault));
    }

    function test_Whitelisted_AlreadyWhitelisted() public {
        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        assertFalse(router.isVaultWhitelisted(vault));

        vm.startPrank(owner);
        router.setVaultWhitelist(vault, true);
        assertTrue(router.isVaultWhitelisted(vault));

        vm.expectRevert("Vault already in desired state");
        router.setVaultWhitelist(vault, true);
        assertTrue(router.isVaultWhitelisted(vault));

        router.setVaultWhitelist(vault, false);
        assertFalse(router.isVaultWhitelisted(vault));
        vm.stopPrank();
    }

    function test_Whitelisted_ExceedsMaxVaultsPerOperator() public {
        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        for (uint256 i = 0; i < 10; i++) {
            vm.prank(operator);
            address vault = address(vaultFactory.create(underlying));
            vm.prank(owner);
            router.setVaultWhitelist(vault, true);
            assertTrue(router.isVaultWhitelisted(vault));
        }

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));

        vm.prank(owner);

        vm.expectRevert("Exceeds max vaults per operator");
        router.setVaultWhitelist(vault, true);

        assertFalse(router.isVaultWhitelisted(vault));
    }

    function test_Whitelisted_NewVaultsCanBeAddedAfterRemoval() public {
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        address[] memory vaults = new address[](10);

        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        for (uint256 i = 0; i < 10; i++) {
            vm.prank(operator);
            vaults[i] = address(vaultFactory.create(underlying));

            vm.prank(owner);
            router.setVaultWhitelist(vaults[i], true);
        }

        vm.prank(operator);
        address newVault = address(vaultFactory.create(underlying));
        assertFalse(router.isVaultWhitelisted(newVault));

        vm.prank(owner);
        router.setVaultWhitelist(vaults[0], false);
        assertFalse(router.isVaultWhitelisted(vaults[0]));

        vm.prank(owner);
        router.setVaultWhitelist(newVault, true);
        assertTrue(router.isVaultWhitelisted(newVault));
    }

    function test_OnlyOwnerCanSetWhitelist() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.setVaultWhitelist(address(0), true);
    }

    function test_setMaxVaultsPerOperator() public {
        vm.prank(owner);
        router.setMaxVaultsPerOperator(20);
        assertEq(router.getMaxVaultsPerOperator(), 20);

        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        for (uint256 i = 0; i < 20; i++) {
            vm.prank(operator);
            address vault = address(vaultFactory.create(underlying));

            vm.prank(owner);
            router.setVaultWhitelist(vault, true);
        }

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));

        vm.prank(owner);
        vm.expectRevert("Exceeds max vaults per operator");
        router.setVaultWhitelist(vault, true);
        assertFalse(router.isVaultWhitelisted(vault));
    }

    function test_setMaxVaultsPerOperator_OnlyOwner() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.setMaxVaultsPerOperator(20);
    }

    function test_setMaxVaultsPerOperator_MustBeGreaterThanCurrent() public {
        vm.prank(owner);
        vm.expectRevert("Must be greater than current");
        router.setMaxVaultsPerOperator(0);
    }

    function test_setMaxVaultsPerOperator_InitialValue() public {
        assertEq(router.getMaxVaultsPerOperator(), 10);

        vm.prank(owner);
        router.setMaxVaultsPerOperator(15);
        assertEq(router.getMaxVaultsPerOperator(), 15);
    }
}
