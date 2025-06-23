// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "../src/InitialImpl.sol";
import "../src/SLAYVault.sol";
import "../src/SLAYVaultFactory.sol";
import "../src/SLAYRouter.sol";
import "../src/SLAYRegistry.sol";

import {Script, console} from "forge-std/Script.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {Options} from "@openzeppelin/foundry-upgrades/Options.sol";
import {Core} from "@openzeppelin/foundry-upgrades/internal/Core.sol";

/// @title Deployment Script for Initialization of SatLayer Protocol
/// @dev For deployment, we use the OpenZeppelin `UnsafeUpgrades` library to deploy UUPS proxies and beacons.
/// Although it is "unsafe" and not recommended for production, the "safe version" does not support non-empty constructor arguments.
/// This "unsafe" allow us to use the constructor arguments in the implementation contracts.
/// Which we use to set immutable proxy addresses for the router and registry.
/// After which we can upgrade the proxies to the actual implementations.
/// However, to ensure the safety of the deployment, we validate each implementation (just as the "safe" version does)
/// to ensure the implementation is valid and does not contain any unsafe code.
contract DeploymentScript is Script {
    Options public opts;
    address public owner = address(0x011);

    function run() public {
        vm.startBroadcast(owner);

        // Create the initial implementation contract and deploy the proxies for router and registry
        Core.validateImplementation("InitialImpl.sol:InitialImpl", opts);
        address initialImpl = address(new InitialImpl());

        SLAYRouter router =
            SLAYRouter(UnsafeUpgrades.deployUUPSProxy(initialImpl, abi.encodeCall(InitialImpl.initialize, (owner))));
        SLAYRegistry registry =
            SLAYRegistry(UnsafeUpgrades.deployUUPSProxy(initialImpl, abi.encodeCall(InitialImpl.initialize, (owner))));

        Core.validateImplementation("SLAYVault.sol:SLAYVault", opts);
        address vaultImpl = address(new SLAYVault(router, registry));
        address beacon = UnsafeUpgrades.deployBeacon(vaultImpl, owner);

        Core.validateImplementation("SLAYVaultFactory.sol:SLAYVaultFactory", opts);
        address factoryImpl = address(new SLAYVaultFactory(beacon, registry));
        UnsafeUpgrades.deployUUPSProxy(factoryImpl, abi.encodeCall(SLAYVaultFactory.initialize, (owner)));

        Core.validateUpgrade("SLAYRouter.sol:SLAYRouter", opts);
        address routerImpl = address(new SLAYRouter(registry));
        UnsafeUpgrades.upgradeProxy(address(router), routerImpl, abi.encodeCall(SLAYRouter.initialize2, ()));

        Core.validateUpgrade("SLAYRegistry.sol:SLAYRegistry", opts);
        address registryImpl = address(new SLAYRegistry(router));
        UnsafeUpgrades.upgradeProxy(address(registry), registryImpl, abi.encodeCall(SLAYRegistry.initialize2, ()));
    }
}
