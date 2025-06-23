// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVault.sol";
import "../src/SLAYVaultFactory.sol";
import {Test, console} from "forge-std/Test.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultFactoryTest is Test, TestSuite {
    MockERC20 public underlying = new MockERC20("Token", "TKN", 18);
    address public immutable operator = makeAddr("Operator X");

    function setUp() public override {
        TestSuite.setUp();

        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator X");
    }

    function test_create_token1() public {
        MockERC20 asset = new MockERC20("Mock Token", "MTK", 8);

        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(asset);

        assertEq(vault.delegated(), operator);
        assertEq(vault.name(), "SatLayer Mock Token");
        assertEq(vault.symbol(), "satMTK");
        assertEq(vault.decimals(), 8);
    }

    function test_create_token2() public {
        MockERC20 asset = new MockERC20("Mock Bit Dollar", "BDR", 15);

        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(asset);

        assertEq(vault.delegated(), operator);
        assertEq(vault.decimals(), 15);
        assertEq(vault.name(), "SatLayer Mock Bit Dollar");
        assertEq(vault.symbol(), "satBDR");
    }

    function test_create_without_metadata() public {
        vm.prank(owner);
        SLAYVault vault = vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");

        assertEq(vault.delegated(), operator);
        assertEq(vault.name(), "Custom Name");
        assertEq(vault.symbol(), "Custom Symbol");
        assertEq(vault.decimals(), 18);
    }

    function test_create_with_not_owner() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");

        vm.startPrank(operator);
        vm.expectRevert(
            abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(operator))
        );
        vaultFactory.create(underlying, operator, "Name", "Symbol");
    }

    function test_create_with_operator() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        assertEq(vault.operator(), operator);
    }

    function test_create_with_not_operator() public {
        vm.expectRevert(abi.encodeWithSelector(SLAYVaultFactory.NotOperator.selector, address(this)));
        vaultFactory.create(underlying);

        address notOperator = makeAddr("Not Operator");
        vm.startPrank(owner);
        vm.expectRevert(abi.encodeWithSelector(SLAYVaultFactory.NotOperator.selector, address(notOperator)));
        vaultFactory.create(underlying, notOperator, "Name", "Symbol");
    }
}
