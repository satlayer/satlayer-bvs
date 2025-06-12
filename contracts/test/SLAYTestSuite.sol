// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {SLAYRouter} from "../src/SLAYRouter.sol";
import {SLAYRegistry} from "../src/SLAYRegistry.sol";
import {EmptyImpl} from "../src/EmptyImpl.sol";

contract SLAYTestSuite is Test {
    address public owner = vm.randomAddress();

    EmptyImpl public emptyImpl = new EmptyImpl();

    SLAYRouter public router;
    SLAYRegistry public registry;

    function setUp() public {
        bytes memory emptyData = abi.encodeCall(EmptyImpl.initialize, (owner));

        router = SLAYRouter(UnsafeUpgrades.deployUUPSProxy(address(emptyImpl), emptyData));
        registry = SLAYRegistry(UnsafeUpgrades.deployUUPSProxy(address(emptyImpl), emptyData));

        vm.startPrank(owner);
        UnsafeUpgrades.upgradeProxy(
            address(router), address(new SLAYRouter(registry)), abi.encodeCall(SLAYRouter.initialize, ())
        );
        UnsafeUpgrades.upgradeProxy(
            address(registry), address(new SLAYRegistry(router)), abi.encodeCall(SLAYRegistry.initialize, ())
        );
        vm.stopPrank();
    }
}
