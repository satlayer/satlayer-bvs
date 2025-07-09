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
        vm.startPrank(makeAddr("Operator Y"));
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        address vault = address(vaultFactory.create(underlying));
        assertFalse(router.whitelisted(vault));

        vm.startPrank(owner);

        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(vault, true);
        router.setVaultWhitelist(vault, true);

        assertTrue(router.whitelisted(vault));

        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(vault, false);
        router.setVaultWhitelist(vault, false);

        assertFalse(router.whitelisted(vault));
    }

    /**
     * We allow whitelisting of fake vaults, which are not created by the factory.
     */
    function test_WhitelistedFakeVault() public {
        address vault = vm.randomAddress();
        assertFalse(router.whitelisted(vault));

        vm.startPrank(owner);

        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(vault, true);
        router.setVaultWhitelist(vault, true);

        assertTrue(router.whitelisted(vault));
    }
}
