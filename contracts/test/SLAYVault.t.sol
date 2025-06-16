// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {SLAYVault} from "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultTest is Test, TestSuite {
    function testToken1() public {
        MockERC20 underlying = new MockERC20("Mock Token", "MTK", 12);
        address proxy = vaultFactory.create(underlying, "SLAY TokenName", "SLAY.MTK");
        SLAYVault vault = SLAYVault(proxy);
        assertEq(vault.decimals(), 12);
        assertEq(vault.symbol(), "SLAY.MTK");
    }

    function testToken2() public {
        MockERC20 underlying = new MockERC20("Mock Token AAA", "AAA", 15);
        address proxy = vaultFactory.create(underlying, "SLAY AAA", "SLAY.AAA");
        SLAYVault vault = SLAYVault(proxy);
        assertEq(vault.decimals(), 15);
        assertEq(vault.symbol(), "SLAY.AAA");
    }
}
