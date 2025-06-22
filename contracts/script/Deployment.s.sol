// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "../src/InitialImpl.sol";
import "../src/SLAYVault.sol";
import "../src/SLAYVaultFactory.sol";
import "../src/SLAYRouter.sol";
import "../src/SLAYRegistry.sol";

import {Script, console} from "forge-std/Script.sol";
import {Upgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";

contract DeploymentScript is Script {
    address public owner = 0x011;

    function run() public {
        InitialImpl initialImpl = new InitialImpl();

        bytes memory initialData = abi.encodeCall(InitialImpl.initialize, (owner));

        SLAYRouter router = SLAYRouter(UnsafeUpgrades.deployUUPSProxy(address(initialImpl), initialData));
        SLAYRegistry registry = SLAYRegistry(UnsafeUpgrades.deployUUPSProxy(address(initialImpl), initialData));

        SLAYVault vaultImpl = new SLAYVault(router, registry);
        address beacon = Upgrades.deployBeacon(address(vaultImpl), owner);
        SLAYVaultFactory factoryImpl = new SLAYVaultFactory(beacon, registry);
        SLAYVaultFactory vaultFactory = SLAYVaultFactory(
            Upgrades.deployUUPSProxy(address(factoryImpl), abi.encodeCall(SLAYVaultFactory.initialize, (owner)))
        );

        vm.startPrank(owner);
        Upgrades.upgradeProxy(
            address(router), address(new SLAYRouter(registry)), abi.encodeCall(SLAYRouter.initialize, ())
        );
        Upgrades.upgradeProxy(
            address(registry), address(new SLAYRegistry(router)), abi.encodeCall(SLAYRegistry.initialize, ())
        );
        vm.stopPrank();
    }
}
