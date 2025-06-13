// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./MockERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {SLAYVault} from "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultTest is Test, TestSuite {
    function testDecimals() public {
        MockERC20 underlying = new MockERC20("Mock Token", "MTK", 12);
        address proxy = vaultFactory.create(underlying, "SLAY TokenName", "SLAY.MTK");
        SLAYVault vault = SLAYVault(proxy);
        assertEq(vault.decimals(), 12);
    }
}
