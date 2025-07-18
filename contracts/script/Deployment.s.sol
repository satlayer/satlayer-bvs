// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {SLAYBase} from "../src/SLAYBase.sol";
import {SLAYVaultV2} from "../src/SLAYVaultV2.sol";
import {SLAYVaultFactoryV2} from "../src/SLAYVaultFactoryV2.sol";
import {SLAYRouterV2} from "../src/SLAYRouterV2.sol";
import {SLAYRegistryV2} from "../src/SLAYRegistryV2.sol";

import {Script, console} from "forge-std/Script.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {Options} from "@openzeppelin/foundry-upgrades/Options.sol";
import {Core} from "@openzeppelin/foundry-upgrades/internal/Core.sol";

/// @title SLAYDeployment Script for Initialization of SatLayer Protocol
/// @dev For deployment, we use the OpenZeppelin `UnsafeUpgrades` library to deploy UUPS proxies and beacons.
/// Although it is "unsafe" and not recommended for production, the "safe version" does not support non-empty constructor arguments.
/// This "unsafe" allow us to use the constructor arguments in the implementation contracts.
/// Which we use to set immutable proxy addresses for the router and registry.
/// After which we can upgrade the proxies to the actual implementations.
/// However, to ensure the safety of the deployment, we validate each implementation (just as the "safe" version does)
/// to ensure the implementation is valid and does not contain any unsafe code.
contract SLAYDeployment is Script {
    Options public opts;

    function run() public virtual {
        address owner = vm.getWallets()[0];
        vm.startBroadcast(owner);
        deploy(owner);
    }

    /// forge script SLAYDeployment --rpc-url slaynet --slow --broadcast --verify
    function deploy(address owner) public {
        console.log("Owner:", owner);

        // Create the initial implementation contract and deploy the proxies for router and registry
        Core.validateImplementation("SLAYBase.sol:SLAYBase", opts);
        address baseImpl = address(new SLAYBase());

        SLAYRouterV2 router =
            SLAYRouterV2(UnsafeUpgrades.deployUUPSProxy(baseImpl, abi.encodeCall(SLAYBase.initialize, (owner))));
        SLAYRegistryV2 registry =
            SLAYRegistryV2(UnsafeUpgrades.deployUUPSProxy(baseImpl, abi.encodeCall(SLAYBase.initialize, (owner))));
        SLAYVaultFactoryV2 vaultFactory =
            SLAYVaultFactoryV2(UnsafeUpgrades.deployUUPSProxy(baseImpl, abi.encodeCall(SLAYBase.initialize, (owner))));

        Core.validateImplementation("SLAYVaultV2.sol:SLAYVaultV2", opts);
        address vaultImpl = address(new SLAYVaultV2(router, registry));
        address beacon = UnsafeUpgrades.deployBeacon(vaultImpl, owner);

        Core.validateUpgrade("SLAYRouterV2.sol:SLAYRouterV2", opts);
        address routerImpl = address(new SLAYRouterV2(registry));
        UnsafeUpgrades.upgradeProxy(address(router), routerImpl, abi.encodeCall(SLAYRouterV2.initialize2, ()));

        Core.validateUpgrade("SLAYRegistryV2.sol:SLAYRegistryV2", opts);
        address registryImpl = address(new SLAYRegistryV2(router));
        UnsafeUpgrades.upgradeProxy(address(registry), registryImpl, abi.encodeCall(SLAYRegistryV2.initialize2, ()));

        Core.validateImplementation("SLAYVaultFactoryV2.sol:SLAYVaultFactoryV2", opts);
        address vaultFactoryImpl = address(new SLAYVaultFactoryV2(beacon, registry));
        UnsafeUpgrades.upgradeProxy(address(vaultFactory), vaultFactoryImpl, "");
    }
}

/// To deploy on SLAYNet.
contract SLAYNetDeployment is SLAYDeployment {
    /// export TENDERLY_RPC_URL=
    /// export TENDERLY_ACCESS_KEY=
    /// forge script SLAYDeployment --rpc-url slaynet --slow --broadcast --verify
    function run() public override {
        uint256 privateKey = vm.randomUint();
        address owner = vm.addr(privateKey);

        string memory params = string(abi.encodePacked('["', vm.toString(owner), '", "0xDE0B6B3A7640000"]'));
        bytes memory result = vm.rpc("tenderly_setBalance", params);
        require(result.length > 0, "Failed to set balance on Tenderly");

        vm.startBroadcast(privateKey);
        super.deploy(owner);
    }
}
