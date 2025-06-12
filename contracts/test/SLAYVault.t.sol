// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./MockERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {SLAYVault} from "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";

contract SLAYVaultTest is Test {
    MockERC20 public underlying = new MockERC20("Mock Token", "MTK", 12);
    SLAYVault public vault;

    function setUp() public {
        SLAYVault implementation = new SLAYVault();
        address proxy = UnsafeUpgrades.deployUUPSProxy(
            address(implementation), abi.encodeCall(SLAYVault.initialize, (underlying, "SLAY TokenName", "SLAY.MTK"))
        );
        vault = SLAYVault(proxy);
    }

    function testDecimals() public {
        assertEq(vault.decimals(), 12);
    }
}
