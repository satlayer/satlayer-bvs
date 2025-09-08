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

abstract contract UpdateScript is Script {
    Options public opts;

    // Upgradeable Proxies: https://build.satlayer.xyz/evm/deployments
    SLAYRouterV2 public router_;
    SLAYRegistryV2 public registry_;
    SLAYVaultFactoryV2 public vaultFactory_;
    SLAYRewardsV2 public rewards_;
    UpgradeableBeacon public beacon_;
    address public signer_;

    function run() public virtual {
        if (block.chainid == 11155111) {
            router_ = SLAYRouterV2(0xfcFd849e8F28EA01b4B7A9Bb170C63C4DF3A8466);
            registry_ = SLAYRegistryV2(0xc44031bc45F39E3B7221F75cfa6BC3589636C622);
            vaultFactory_ = SLAYVaultFactoryV2(0xcC0Ede3FB35F783095f8823f27111c5a7E573973);
            rewards_ = SLAYRewardsV2(0x06A1dDDc822aa56D80E50A82Be9154b53f07865b);
            beacon_ = UpgradeableBeacon(0x357c341eE5766887013A2F4197b2E6D1045c477c);

            signer_ = 0x511913AdB7e16e449a1219a00E8c274B3CFbd754;
        } else {
            revert("Chain upgrade not supported");
        }

        // TODO(fuxingloh): for mainnet pausing will be performed.

        upgrade();
    }

    function upgrade() public virtual;
}

contract UpgradeSLAYRouterV2_X is UpdateScript {
    function upgrade() public override {
        Core.validateUpgrade("SLAYRouterV2.sol:SLAYRouterV2", opts);
        vm.startBroadcast();
        address routerImpl = address(new SLAYRouterV2(registry_));
        vm.stopBroadcast();

        vm.startPrank(signer_);
        Core.upgradeProxyTo(address(router_), routerImpl, "");
    }
}

contract UpgradeSLAYRegistryV2_X is UpdateScript {
    function upgrade() public override {
        Core.validateUpgrade("SLAYRegistryV2.sol:SLAYRegistryV2", opts);
        vm.startBroadcast();
        address registryImpl = address(new SLAYRegistryV2(router_));
        vm.stopBroadcast();

        vm.startPrank(signer_);
        Core.upgradeProxyTo(address(registry_), registryImpl, "");
    }
}

contract UpgradeSLAYVaultFactoryV2_X is UpdateScript {
    function upgrade() public override {
        Core.validateUpgrade("SLAYVaultFactoryV2.sol:SLAYVaultFactoryV2", opts);
        vm.startBroadcast();
        address vaultFactoryImpl = address(new SLAYVaultFactoryV2(address(beacon_), registry_));
        vm.stopBroadcast();

        vm.startPrank(signer_);
        Core.upgradeProxyTo(address(vaultFactory_), vaultFactoryImpl, "");
    }
}

contract UpgradeSLAYRewardsV2_X is UpdateScript {
    function upgrade() public override {
        Core.validateUpgrade("SLAYRewardsV2.sol:SLAYRewardsV2", opts);
        vm.startBroadcast();
        address rewardsImpl = address(new SLAYRewardsV2());
        vm.stopBroadcast();

        vm.startPrank(signer_);
        Core.upgradeProxyTo(address(rewards_), rewardsImpl, "");
    }
}

contract UpgradeSLAYVaultV2_X is UpdateScript {
    function upgrade() public override {
        // Disabled for now
        // Core.validateUpgrade("SLAYVaultV2.sol:SLAYVaultV2", opts);
        vm.startBroadcast();
        address vaultImpl = address(new SLAYVaultV2(router_, registry_));
        vm.stopBroadcast();

        vm.startPrank(signer_);
        Core.upgradeBeaconTo(address(beacon_), vaultImpl);
    }
}
