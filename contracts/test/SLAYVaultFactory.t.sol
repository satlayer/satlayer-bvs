// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultFactoryTest is Test, TestSuite {
    address public immutable operator = makeAddr("Operator X");

    function setUp() public override {
        TestSuite.setUp();

        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator X");
    }

    function test_create_token1() public {
        MockERC20 underlying = new MockERC20("Mock Token", "MTK", 8);

        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        assertEq(vault.operator(), operator);
        assertEq(vault.name(), "SatLayer Mock Token");
        assertEq(vault.symbol(), "satMTK");
        assertEq(vault.decimals(), 8);
    }

    function test_create_token2() public {
        MockERC20 underlying = new MockERC20("Mock Bit Dollar", "BDR", 15);

        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        assertEq(vault.operator(), operator);
        assertEq(vault.decimals(), 15);
        assertEq(vault.name(), "SatLayer Mock Bit Dollar");
        assertEq(vault.symbol(), "satBDR");
    }

    function test_create_without_metadata() public {
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        vm.prank(owner);
        SLAYVault vault = vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");

        assertEq(vault.operator(), operator);
        assertEq(vault.name(), "Custom Name");
        assertEq(vault.symbol(), "Custom Symbol");
        assertEq(vault.decimals(), 18);
    }

    function test_create_with_not_owner() public {
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");

        vm.startPrank(operator);
        vm.expectRevert(
            abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(operator))
        );
        vaultFactory.create(underlying, operator, "Name", "Symbol");
    }
}
