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
        assertFalse(router.whitelisted(vault));

        vm.prank(owner);
        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(operator, vault, true);
        router.setVaultWhitelist(vault, true);

        assertTrue(router.whitelisted(vault));

        vm.prank(owner);
        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(operator, vault, false);
        router.setVaultWhitelist(vault, false);

        assertFalse(router.whitelisted(vault));
    }

    function test_Whitelisted_NotVault() public {
        address vault = vm.randomAddress();
        assertFalse(router.whitelisted(vault));

        vm.prank(owner);
        vm.expectRevert();
        router.setVaultWhitelist(vault, true);

        assertFalse(router.whitelisted(vault));
    }

    function test_Whitelisted_AlreadyWhitelisted() public {
        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        assertFalse(router.whitelisted(vault));

        vm.startPrank(owner);
        router.setVaultWhitelist(vault, true);
        assertTrue(router.whitelisted(vault));

        vm.expectRevert("Vault already in desired state");
        router.setVaultWhitelist(vault, true);
        assertTrue(router.whitelisted(vault));

        router.setVaultWhitelist(vault, false);
        assertFalse(router.whitelisted(vault));
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
            assertTrue(router.whitelisted(vault));
        }

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));

        vm.prank(owner);

        vm.expectRevert("Exceeds max vaults per operator");
        router.setVaultWhitelist(vault, true);

        assertFalse(router.whitelisted(vault));
    }
}
