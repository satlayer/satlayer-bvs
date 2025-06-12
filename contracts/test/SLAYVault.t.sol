// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./MockERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {SLAYVault} from "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {Upgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";

contract SLAYVaultTest is Test {
    address public proxy;
    MockERC20 public token;
    SLAYVault public vault;

    function setUp() public {
        token = new MockERC20("Mock Token", "MTK", 18);
        proxy = Upgrades.deployUUPSProxy(
            "SLAYVault.sol:SLAYVault", abi.encodeCall(SLAYVault.initialize, (token, "SLAY TokenName", "SLAY.MTK"))
        );
        vault = SLAYVault(proxy);
    }

    function testDecimals() public {
        assertEq(vault.decimals(), 18);
    }
}
