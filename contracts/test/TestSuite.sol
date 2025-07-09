// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {MockERC20} from "./MockERC20.sol";
import {InitialV1} from "../src/InitialV1.sol";
import {SLAYRegistry} from "../src/SLAYRegistry.sol";
import {SLAYRouter} from "../src/SLAYRouter.sol";
import {SLAYVaultFactory} from "../src/SLAYVaultFactory.sol";
import {SLAYVault} from "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";

/**
 * @dev This test suite set up all the contracts needed for testing.
 */
contract TestSuite is Test {
    address public owner = vm.randomAddress();

    InitialV1 public initialImpl = new InitialV1();

    SLAYRouter public router;
    SLAYRegistry public registry;
    SLAYVaultFactory public vaultFactory;

    function setUp() public virtual {
        bytes memory initialData = abi.encodeCall(InitialV1.initialize, (owner));

        router = SLAYRouter(UnsafeUpgrades.deployUUPSProxy(address(initialImpl), initialData));
        registry = SLAYRegistry(UnsafeUpgrades.deployUUPSProxy(address(initialImpl), initialData));

        SLAYVault vaultImpl = new SLAYVault(router, registry);
        address beacon = UnsafeUpgrades.deployBeacon(address(vaultImpl), owner);
        SLAYVaultFactory factoryImpl = new SLAYVaultFactory(beacon, registry);
        vaultFactory = SLAYVaultFactory(
            UnsafeUpgrades.deployUUPSProxy(address(factoryImpl), abi.encodeCall(SLAYVaultFactory.initialize, (owner)))
        );

        vm.startPrank(owner);
        UnsafeUpgrades.upgradeProxy(address(router), address(new SLAYRouter(registry)), "");
        UnsafeUpgrades.upgradeProxy(
            address(registry), address(new SLAYRegistry(router)), abi.encodeCall(SLAYRegistry.initialize, ())
        );
        vm.stopPrank();
    }
}
