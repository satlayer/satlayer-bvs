// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultTest is Test, TestSuite {
    function test_create_token1() public {
        MockERC20 underlying = new MockERC20("Mock Token", "MTK", 8);

        address operator = vm.randomAddress();
        vm.startPrank(operator);
        address proxy = vaultFactory.create(underlying);
        vm.stopPrank();

        SLAYVault vault = SLAYVault(proxy);
        assertEq(vault.operator(), operator);
        assertEq(vault.name(), "SatLayer Mock Token");
        assertEq(vault.symbol(), "satMTK");
        assertEq(vault.decimals(), 8);
    }

    function test_create_token2() public {
        MockERC20 underlying = new MockERC20("Mock Bit Dollar", "BDR", 15);

        address operator = vm.randomAddress();
        vm.startPrank(operator);
        address proxy = vaultFactory.create(underlying);
        vm.stopPrank();

        SLAYVault vault = SLAYVault(proxy);
        assertEq(vault.operator(), operator);
        assertEq(vault.decimals(), 15);
        assertEq(vault.name(), "SatLayer Mock Bit Dollar");
        assertEq(vault.symbol(), "satBDR");
    }

    function test_create_without_metadata() public {
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        address operator = vm.randomAddress();

        vm.startPrank(owner);
        address proxy = vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");
        vm.stopPrank();

        SLAYVault vault = SLAYVault(proxy);
        assertEq(vault.operator(), operator);
        assertEq(vault.name(), "Custom Name");
        assertEq(vault.symbol(), "Custom Symbol");
        assertEq(vault.decimals(), 18);
    }

    function test_create_with_not_owner() public {
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        address operator = vm.randomAddress();
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");

        vm.startPrank(operator);
        vm.expectRevert(
            abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(operator))
        );
        vaultFactory.create(underlying, operator, "Name", "Symbol");
    }
}
