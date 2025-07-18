// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {SLAYBase} from "../src/SLAYBase.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

contract SLAYBaseTest is Test {
    SLAYBase public initialImpl = new SLAYBase();

    function test_paused() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
        SLAYBase impl = SLAYBase(proxy);
        impl.pause();

        assertTrue(impl.paused(), "Contract should be paused");
    }

    function test_unpaused() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
        SLAYBase impl = SLAYBase(proxy);
        impl.pause();
        impl.unpause();

        assertFalse(impl.paused(), "Contract should be unpaused");
    }

    function test_owner() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
        SLAYBase impl = SLAYBase(proxy);

        address owner = address(this);
        assertEq(impl.owner(), owner, "Owner should be set correctly");
    }

    function test_only_owner_can_pause() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
        SLAYBase impl = SLAYBase(proxy);

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(0x123)));
        vm.prank(address(0x123));
        impl.pause();
    }

    function test_only_owner_can_unpause() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
        SLAYBase impl = SLAYBase(proxy);
        impl.pause();

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(0x123)));
        vm.prank(address(0x123));
        impl.unpause();
    }

    function test_upgrade_proxy() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
        SLAYBase impl = SLAYBase(proxy);

        SLAYBase newImpl = new SLAYBase();
        UnsafeUpgrades.upgradeProxy(proxy, address(newImpl), "");

        assertEq(impl.owner(), address(this), "Owner should remain unchanged after upgrade");
    }

    function test_only_owner_can_upgrade() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));

        SLAYBase newImpl = new SLAYBase();
        try this.upgradeCallNonOwner(proxy, address(newImpl)) {
            fail();
        } catch (bytes memory reason) {
            assertEq(
                reason, abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(0x123))
            );
        }
    }

    /// For {test_only_owner_can_upgrade}
    function upgradeCallNonOwner(address proxy, address newImpl) external {
        vm.startPrank(address(0x123));
        UnsafeUpgrades.upgradeProxy(proxy, newImpl, "");
    }

    function test_initializable() public {
        UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
    }

    function test_initializable_fails_if_initialized_twice() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
        SLAYBase impl = SLAYBase(proxy);

        vm.expectRevert(Initializable.InvalidInitialization.selector);
        impl.initialize(address(this));
    }

    function test_pause_post_upgrade_still_paused() public {
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (address(this))));
        SLAYBase impl = SLAYBase(proxy);
        impl.pause();

        SLAYBase newImpl = new SLAYBase();
        UnsafeUpgrades.upgradeProxy(proxy, address(newImpl), "");

        assertTrue(impl.paused(), "Contract should still be paused after upgrade");
    }
}
