// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "../src/SLAYRouter.sol";
import "../src/SLAYVault.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYRouterTest is Test, TestSuite {
    function test_defaults() public view {
        assertEq(router.owner(), owner);
        assertEq(router.paused(), false);
    }

    function test_paused() public {
        vm.startPrank(owner);
        router.pause();
        vm.stopPrank();

        assertTrue(router.paused());
    }

    function test_pausedOnlyOwnerError() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.pause();
    }

    function test_unpausedOnlyOwnerError() public {
        vm.startPrank(owner);
        router.pause();
        vm.stopPrank();

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.unpause();
    }

    function test_Whitelisted() public {
        address vault = address(newVault());
        assertFalse(router.whitelisted(vault));

        vm.startPrank(owner);

        vm.expectEmit();
        emit SLAYRouter.Whitelisted(vault, true);
        router.setWhitelist(vault, true);

        assertTrue(router.whitelisted(vault));

        vm.expectEmit();
        emit SLAYRouter.Whitelisted(vault, false);
        router.setWhitelist(vault, false);

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
        emit SLAYRouter.Whitelisted(vault, true);
        router.setWhitelist(vault, true);

        assertTrue(router.whitelisted(vault));
    }
}
