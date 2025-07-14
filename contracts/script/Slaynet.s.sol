// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {InitialImpl} from "../src/InitialImpl.sol";
import {SLAYVaultV2} from "../src/SLAYVaultV2.sol";
import {SLAYVaultFactoryV2} from "../src/SLAYVaultFactoryV2.sol";
import {SLAYRouterV2} from "../src/SLAYRouterV2.sol";
import {SLAYRegistryV2} from "../src/SLAYRegistryV2.sol";

import {Script, console} from "forge-std/Script.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {Options} from "@openzeppelin/foundry-upgrades/Options.sol";
import {Core} from "@openzeppelin/foundry-upgrades/internal/Core.sol";

/// @title Slaynet Deployment Script for Initialization of SatLayer Protocol
/// @dev For deployment, we use the OpenZeppelin `UnsafeUpgrades` library to deploy UUPS proxies and beacons.
/// Although it is "unsafe" and not recommended for production, the "safe version" does not support non-empty constructor arguments.
/// This "unsafe" allow us to use the constructor arguments in the implementation contracts.
/// Which we use to set immutable proxy addresses for the router and registry.
/// After which we can upgrade the proxies to the actual implementations.
/// However, to ensure the safety of the deployment, we validate each implementation (just as the "safe" version does)
/// to ensure the implementation is valid and does not contain any unsafe code.
contract SlaynetDeployment is Script {
    Options public opts;

    /// export PRIVATE_KEY=
    /// export TENDERLY_RPC_URL=
    /// export TENDERLY_ACCESS_KEY=
    /// forge script SlaynetDeployment --rpc-url slaynet --slow --broadcast --verify
    function run() public {
        uint256 pk = vm.envUint("PRIVATE_KEY");
        address owner = vm.addr(pk);
        vm.startBroadcast(pk);

        // Create the initial implementation contract and deploy the proxies for router and registry
        Core.validateImplementation("InitialImpl.sol:InitialImpl", opts);
        address initialImpl = address(new InitialImpl());

        SLAYRouterV2 router =
            SLAYRouterV2(UnsafeUpgrades.deployUUPSProxy(initialImpl, abi.encodeCall(InitialImpl.initialize, (owner))));
        SLAYRegistryV2 registry =
            SLAYRegistryV2(UnsafeUpgrades.deployUUPSProxy(initialImpl, abi.encodeCall(InitialImpl.initialize, (owner))));

        Core.validateImplementation("SLAYVaultV2.sol:SLAYVaultV2", opts);
        address vaultImpl = address(new SLAYVaultV2(router, registry));
        address beacon = UnsafeUpgrades.deployBeacon(vaultImpl, owner);

        Core.validateImplementation("SLAYVaultFactoryV2.sol:SLAYVaultFactoryV2", opts);
        address factoryImpl = address(new SLAYVaultFactoryV2(beacon, registry));
        UnsafeUpgrades.deployUUPSProxy(factoryImpl, abi.encodeCall(SLAYVaultFactoryV2.initialize, (owner)));

        Core.validateUpgrade("SLAYRouterV2.sol:SLAYRouterV2", opts);
        address routerImpl = address(new SLAYRouterV2(registry));
        UnsafeUpgrades.upgradeProxy(address(router), routerImpl, abi.encodeCall(SLAYRouterV2.initialize2, ()));

        Core.validateUpgrade("SLAYRegistryV2.sol:SLAYRegistryV2", opts);
        address registryImpl = address(new SLAYRegistryV2(router));
        UnsafeUpgrades.upgradeProxy(address(registry), registryImpl, abi.encodeCall(SLAYRegistryV2.initialize2, ()));
    }
}
