pragma solidity ^0.8.20;

import {MockERC20} from "./MockERC20.sol";
import {SLAYBase} from "@satlayer/contracts/SLAYBase.sol";
import {SLAYRegistryV2} from "@satlayer/contracts/SLAYRegistryV2.sol";
import {SLAYRouterV2} from "@satlayer/contracts/SLAYRouterV2.sol";
import {SLAYVaultFactoryV2} from "@satlayer/contracts/SLAYVaultFactoryV2.sol";
import {SLAYVaultV2} from "@satlayer/contracts/SLAYVaultV2.sol";
import {Test, console} from "forge-std/Test.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {SLAYRewardsV2} from "@satlayer/contracts/SLAYRewardsV2.sol";

/**
 * @dev This test suite set up all the V2 contracts needed for testing.
 */
contract TestSuiteV2 is Test {
    address public owner = vm.randomAddress();

    address public baseImpl = address(new SLAYBase());

    SLAYRouterV2 public router;
    SLAYRegistryV2 public registry;
    SLAYVaultFactoryV2 public vaultFactory;
    SLAYRewardsV2 public rewards;

    function setUp() public virtual {
        bytes memory baseInit = abi.encodeCall(SLAYBase.initialize, (owner));

        router = SLAYRouterV2(UnsafeUpgrades.deployUUPSProxy(baseImpl, baseInit));
        registry = SLAYRegistryV2(UnsafeUpgrades.deployUUPSProxy(baseImpl, baseInit));
        vaultFactory = SLAYVaultFactoryV2(UnsafeUpgrades.deployUUPSProxy(baseImpl, baseInit));
        rewards = SLAYRewardsV2(UnsafeUpgrades.deployUUPSProxy(baseImpl, baseInit));

        SLAYVaultV2 vaultImpl = new SLAYVaultV2(router, registry);
        address beacon = UnsafeUpgrades.deployBeacon(address(vaultImpl), owner);

        vm.startPrank(owner);
        UnsafeUpgrades.upgradeProxy(
            address(router), address(new SLAYRouterV2(registry)), abi.encodeCall(SLAYRouterV2.initialize2, ())
        );
        UnsafeUpgrades.upgradeProxy(
            address(registry), address(new SLAYRegistryV2(router)), abi.encodeCall(SLAYRegistryV2.initialize2, ())
        );
        UnsafeUpgrades.upgradeProxy(address(vaultFactory), address(new SLAYVaultFactoryV2(beacon, registry)), "");
        UnsafeUpgrades.upgradeProxy(address(rewards), address(new SLAYRewardsV2()), "");
        vm.stopPrank();
    }

    function _advanceBlockBy(uint256 blocks) internal {
        vm.roll(block.number + blocks);
        vm.warp(block.timestamp + (12 * blocks));
    }

    function _advanceBlockBySeconds(uint256 newSeconds) internal {
        vm.roll(block.number + (newSeconds / 12));
        vm.warp(block.timestamp + newSeconds);
    }
}
