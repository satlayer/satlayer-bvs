// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "../src/InitialImpl.sol";
import "../src/SLAYVault.sol";
import "../src/SLAYVaultFactory.sol";
import "../src/SLAYRouter.sol";
import "../src/SLAYRegistry.sol";

import {Script, console} from "forge-std/Script.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";

/// @title Deployment Script for Initialization of SatLayer Protocol
/// @dev For deployment, we use the OpenZeppelin `UnsafeUpgrades` library to deploy UUPS proxies and beacons.
/// Although it is "unsafe" and not recommended for production, the "safe version" does not support non-empty constructor arguments.
/// This allow us to use the constructor arguments in the implementation contracts.
/// Which we use to set immutable proxy addresses for the router and registry.
/// After which we can upgrade the proxies to the actual implementations.
contract DeploymentScript is Script {
    address public owner = address(0x011);

    function run() public {
        vm.startBroadcast(owner);

        // Create the initial implementation contract and deploy the proxies for router and registry
        address initialImpl = address(new InitialImpl());
        SLAYRouter router =
            SLAYRouter(UnsafeUpgrades.deployUUPSProxy(initialImpl, abi.encodeCall(InitialImpl.initialize, (owner))));
        SLAYRegistry registry =
            SLAYRegistry(UnsafeUpgrades.deployUUPSProxy(initialImpl, abi.encodeCall(InitialImpl.initialize, (owner))));

        address vaultImpl = address(new SLAYVault(router, registry));
        address beacon = UnsafeUpgrades.deployBeacon(vaultImpl, owner);
        address factoryImpl = address(new SLAYVaultFactory(beacon, registry));
        SLAYVaultFactory vaultFactory = SLAYVaultFactory(
            UnsafeUpgrades.deployUUPSProxy(factoryImpl, abi.encodeCall(SLAYVaultFactory.initialize, (owner)))
        );

        address routerImpl = address(new SLAYRouter(registry));
        UnsafeUpgrades.upgradeProxy(address(router), routerImpl, abi.encodeCall(SLAYRouter.initialize, ()));

        address registryImpl = address(new SLAYRegistry(router));
        UnsafeUpgrades.upgradeProxy(address(registry), registryImpl, abi.encodeCall(SLAYRegistry.initialize, ()));
    }
}
