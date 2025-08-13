// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {SLAYBase} from "../src/SLAYBase.sol";
import {SLAYVaultV2} from "../src/SLAYVaultV2.sol";
import {SLAYVaultFactoryV2} from "../src/SLAYVaultFactoryV2.sol";
import {SLAYRouterV2} from "../src/SLAYRouterV2.sol";
import {SLAYRegistryV2} from "../src/SLAYRegistryV2.sol";
import {SLAYRewardsV2} from "../src/SLAYRewardsV2.sol";

import {Script, console} from "forge-std/Script.sol";
import {Options} from "@openzeppelin/foundry-upgrades/Options.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {UpgradeableBeacon} from "@openzeppelin/contracts/proxy/beacon/UpgradeableBeacon.sol";
import {Core} from "@openzeppelin/foundry-upgrades/internal/Core.sol";

/// @title SLAYDeployment Script for Initialization of SatLayer Protocol
/// @dev For deployment, we use the OpenZeppelin foundry-upgrades `Core` library to deploy UUPS proxies and beacons.
/// Although it is "unsafe" and not recommended for production, the "safe version" does not support non-empty constructor arguments.
/// This "unsafe" allow us to use the constructor arguments in the implementation contracts.
/// Which we use to set immutable proxy addresses for the router and registry.
/// After which we can upgrade the proxies to the actual implementations.
/// However, to ensure the safety of the deployment, we validate each implementation (just as the "safe" version does)
/// to ensure the implementation is valid and does not contain any unsafe code.
///
/// IMPORTANT: The only difference between `Core.upgradeProxyTo` and `Upgrades.upgradeProxy` is
/// that the former does not run `Core.validateUpgrade` before running deploy or upgrade.
/// Hence `Core.validateUpgrade` is called manually in this script before each upgrade.
contract SLAYDeployment is Script {
    Options public opts;

    function run() public virtual {
        vm.startBroadcast();
        deploy(msg.sender, address(0));
    }

    /// @dev Deploys the SatLayer Protocol core contracts.
    /// forge script SLAYDeployment --slow --broadcast --verify
    function deploy(address initialOwner, address newOwner)
        public
        returns (SLAYRouterV2 router, SLAYRegistryV2 registry, SLAYVaultFactoryV2 vaultFactory, SLAYRewardsV2 rewards)
    {
        console.log("Initial Owner:", initialOwner);

        // Create the initial implementation contract and deploy the proxies for router and registry
        Core.validateImplementation("SLAYBase.sol:SLAYBase", opts);
        address baseImpl = address(new SLAYBase{salt: keccak256("SatLayer")}());

        // We use the same SLAYBase for all contracts here.
        bytes memory baseInit = abi.encodeCall(SLAYBase.initialize, (initialOwner));

        router = SLAYRouterV2(create2UUPSProxy(baseImpl, baseInit));
        registry = SLAYRegistryV2(create2UUPSProxy(baseImpl, baseInit));
        vaultFactory = SLAYVaultFactoryV2(create2UUPSProxy(baseImpl, baseInit));
        rewards = SLAYRewardsV2(create2UUPSProxy(baseImpl, baseInit));

        Core.validateImplementation("SLAYVaultV2.sol:SLAYVaultV2", opts);
        address vaultImpl = address(new SLAYVaultV2(router, registry));
        UpgradeableBeacon beacon = new UpgradeableBeacon(vaultImpl, initialOwner);

        Core.validateUpgrade("SLAYRouterV2.sol:SLAYRouterV2", opts);
        address routerImpl = address(new SLAYRouterV2(registry));
        Core.upgradeProxyTo(address(router), routerImpl, abi.encodeCall(SLAYRouterV2.initialize2, ()));

        Core.validateUpgrade("SLAYRegistryV2.sol:SLAYRegistryV2", opts);
        address registryImpl = address(new SLAYRegistryV2(router));
        Core.upgradeProxyTo(address(registry), registryImpl, abi.encodeCall(SLAYRegistryV2.initialize2, ()));

        Core.validateUpgrade("SLAYVaultFactoryV2.sol:SLAYVaultFactoryV2", opts);
        address vaultFactoryImpl = address(new SLAYVaultFactoryV2(address(beacon), registry));
        Core.upgradeProxyTo(address(vaultFactory), vaultFactoryImpl, "");

        Core.validateUpgrade("SLAYRewardsV2.sol:SLAYRewardsV2", opts);
        address rewardsImpl = address(new SLAYRewardsV2());
        Core.upgradeProxyTo(address(rewards), rewardsImpl, "");

        // Transfer ownership
        if (initialOwner != newOwner && newOwner != address(0)) {
            beacon.transferOwnership(newOwner);
            router.transferOwnership(newOwner);
            registry.transferOwnership(newOwner);
            vaultFactory.transferOwnership(newOwner);
            rewards.transferOwnership(newOwner);
        }
    }

    function create2UUPSProxy(address impl, bytes memory initializerData) internal returns (address) {
        bytes32 salt = keccak256("SatLayer");
        return address(new ERC1967Proxy{salt: salt}(impl, initializerData));
    }
}
