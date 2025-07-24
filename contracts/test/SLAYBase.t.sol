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

    function test_transfer_ownership() public {
        address initialOwner = makeAddr("Initial Owner");
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (initialOwner)));
        SLAYBase impl = SLAYBase(proxy);

        address newOwner = makeAddr("New Owner");

        // first step to transfer ownership
        vm.prank(initialOwner);
        impl.transferOwnership(newOwner);

        // assert that the ownership is not yet transferred
        assertEq(impl.owner(), initialOwner, "Ownership should not be transferred yet");

        // assert that pending owner is new owner
        assertEq(impl.pendingOwner(), newOwner, "Pending owner should be set to the new owner");

        // second step to transfer ownership
        vm.prank(newOwner);
        impl.acceptOwnership();

        // assert that the ownership is now transferred
        assertEq(impl.owner(), newOwner, "Ownership should be transferred to the new owner");
    }

    function test_transfer_ownership_multiple() public {
        address initialOwner = makeAddr("Initial Owner");
        address proxy =
            UnsafeUpgrades.deployUUPSProxy(address(initialImpl), abi.encodeCall(SLAYBase.initialize, (initialOwner)));
        SLAYBase impl = SLAYBase(proxy);

        address newOwner = makeAddr("New Owner");
        address anotherNewOwner = makeAddr("Another New Owner");

        // first step to transfer ownership
        vm.prank(initialOwner);
        impl.transferOwnership(newOwner);

        // assert that the ownership is not yet transferred
        assertEq(impl.owner(), initialOwner, "Ownership should not be transferred yet");

        // assert that pending owner is new owner
        assertEq(impl.pendingOwner(), newOwner, "Pending owner should be set to the new owner");

        // initial owner changes mind and wants to transfer ownership to another address
        vm.prank(initialOwner);
        impl.transferOwnership(anotherNewOwner);

        // assert that the pending owner is now set to the another new owner
        assertEq(impl.pendingOwner(), anotherNewOwner, "Pending owner should be set to the another new owner");

        // new owner should not be able to accept ownership
        vm.prank(newOwner);
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, newOwner));
        impl.acceptOwnership();

        // another new owner accepts ownership
        vm.prank(anotherNewOwner);
        impl.acceptOwnership();
        // assert that the ownership is now transferred to the another new owner
        assertEq(impl.owner(), anotherNewOwner, "Ownership should be transferred to the another new owner");

        // another new owner can transfer ownership back to the initial owner
        vm.prank(anotherNewOwner);
        impl.transferOwnership(initialOwner);
        // assert that the ownership is not yet transferred
        assertEq(impl.owner(), anotherNewOwner, "Ownership should not be transferred yet");
        // assert that pending owner is initial owner
        assertEq(impl.pendingOwner(), initialOwner, "Pending owner should be set to the initial owner");

        // initial owner accepts ownership
        vm.prank(initialOwner);
        impl.acceptOwnership();
        // assert that the ownership is now transferred back to the initial owner
        assertEq(impl.owner(), initialOwner, "Ownership should be transferred back to the initial owner");
    }
}
