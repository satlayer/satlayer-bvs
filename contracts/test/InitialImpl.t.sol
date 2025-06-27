// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {InitialImpl} from "../src/InitialImpl.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

contract InitialImplTest is Test {
    InitialImpl public initialImpl = new InitialImpl();

    function test_paused() public {
        address proxy = UnsafeUpgrades.deployUUPSProxy(
            address(initialImpl), abi.encodeCall(InitialImpl.initialize, (address(this)))
        );
        InitialImpl impl = InitialImpl(proxy);
        impl.pause();

        assertTrue(impl.paused(), "Contract should be paused");
    }

    function test_unpaused() public {
        address proxy = UnsafeUpgrades.deployUUPSProxy(
            address(initialImpl), abi.encodeCall(InitialImpl.initialize, (address(this)))
        );
        InitialImpl impl = InitialImpl(proxy);
        impl.pause();
        impl.unpause();

        assertFalse(impl.paused(), "Contract should be unpaused");
    }

    function test_owner() public {
        address proxy = UnsafeUpgrades.deployUUPSProxy(
            address(initialImpl), abi.encodeCall(InitialImpl.initialize, (address(this)))
        );
        InitialImpl impl = InitialImpl(proxy);

        address owner = address(this);
        assertEq(impl.owner(), owner, "Owner should be set correctly");
    }

    function test_only_owner_can_pause() public {
        address proxy = UnsafeUpgrades.deployUUPSProxy(
            address(initialImpl), abi.encodeCall(InitialImpl.initialize, (address(this)))
        );
        InitialImpl impl = InitialImpl(proxy);

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(0x123)));
        vm.prank(address(0x123));
        impl.pause();
    }

    function test_only_owner_can_unpause() public {
        address proxy = UnsafeUpgrades.deployUUPSProxy(
            address(initialImpl), abi.encodeCall(InitialImpl.initialize, (address(this)))
        );
        InitialImpl impl = InitialImpl(proxy);
        impl.pause();

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(0x123)));
        vm.prank(address(0x123));
        impl.unpause();
    }

    function test_upgrade_proxy() public {
        address proxy = UnsafeUpgrades.deployUUPSProxy(
            address(initialImpl), abi.encodeCall(InitialImpl.initialize, (address(this)))
        );
        InitialImpl impl = InitialImpl(proxy);

        InitialImpl newImpl = new InitialImpl();
        UnsafeUpgrades.upgradeProxy(proxy, address(newImpl), "");

        assertEq(impl.owner(), address(this), "Owner should remain unchanged after upgrade");
    }

    function test_only_owner_can_upgrade() public {
        address proxy = UnsafeUpgrades.deployUUPSProxy(
            address(initialImpl), abi.encodeCall(InitialImpl.initialize, (address(this)))
        );
        InitialImpl impl = InitialImpl(proxy);

        InitialImpl newImpl = new InitialImpl();
        try this.upgradeCallNonOwner(proxy, address(newImpl)) {
            fail();
        } catch (bytes memory reason) {
            assertEq(
                reason, abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(0x123))
            );
        }
    }

    function upgradeCallNonOwner(address proxy, address newImpl) external {
        vm.startPrank(address(0x123));
        UnsafeUpgrades.upgradeProxy(proxy, newImpl, "");
    }
}
