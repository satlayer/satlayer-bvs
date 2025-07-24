// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVaultV2.sol";
import "../src/SLAYVaultFactoryV2.sol";
import {Test, console} from "forge-std/Test.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {TestSuiteV2} from "./TestSuiteV2.sol";
import {ISLAYVaultFactoryV2} from "../src/interface/ISLAYVaultFactoryV2.sol";

contract SLAYVaultFactoryV2Test is Test, TestSuiteV2 {
    MockERC20 public underlying = new MockERC20("Token", "TKN", 18);
    address public immutable operator = makeAddr("Operator X");

    function setUp() public override {
        TestSuiteV2.setUp();

        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator X");
    }

    function test_create_token1() public {
        MockERC20 asset = new MockERC20("Mock Token", "MTK", 8);

        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(asset);

        assertEq(vault.delegated(), operator);
        assertEq(vault.name(), "SatLayer Mock Token");
        assertEq(vault.symbol(), "satMTK");
        assertEq(vault.decimals(), 8);
    }

    function test_create_token2() public {
        MockERC20 asset = new MockERC20("Mock Bit Dollar", "BDR", 15);

        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(asset);

        assertEq(vault.delegated(), operator);
        assertEq(vault.decimals(), 15);
        assertEq(vault.name(), "SatLayer Mock Bit Dollar");
        assertEq(vault.symbol(), "satBDR");
    }

    function test_create_without_metadata() public {
        vm.prank(owner);
        SLAYVaultV2 vault = vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");

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
        SLAYVaultV2 vault = vaultFactory.create(underlying);

        assertEq(vault.delegated(), operator);
    }

    function test_create_with_not_operator() public {
        vm.expectRevert(abi.encodeWithSelector(ISLAYVaultFactoryV2.NotOperator.selector, address(this)));
        vaultFactory.create(underlying);

        address notOperator = makeAddr("Not Operator");
        vm.startPrank(owner);
        vm.expectRevert(abi.encodeWithSelector(ISLAYVaultFactoryV2.NotOperator.selector, address(notOperator)));
        vaultFactory.create(underlying, notOperator, "Name", "Symbol");
    }

    function test_create_whenPaused() public {
        vm.prank(owner);
        vaultFactory.pause();

        // Try to create a vault when paused
        vm.startPrank(operator);
        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        vaultFactory.create(underlying);
        vm.stopPrank();

        // Try to create a vault with custom params when paused
        vm.startPrank(owner);
        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");
        vm.stopPrank();
    }

    function test_create_whenPausedAndUnpaused() public {
        // Pause the contract
        vm.prank(owner);
        vaultFactory.pause();
        assertTrue(vaultFactory.paused());

        // Unpause the contract
        vm.prank(owner);
        vaultFactory.unpause();
        assertFalse(vaultFactory.paused());

        // Create a vault when unpaused
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);
        assertEq(vault.delegated(), operator);

        // Create a vault with custom params when unpaused
        vm.prank(owner);
        SLAYVaultV2 customVault = vaultFactory.create(underlying, operator, "Custom Name", "Custom Symbol");
        assertEq(customVault.delegated(), operator);
    }

    function test_immutable_beacon() public view {
        // The beacon address should not be zero
        assertTrue(vaultFactory.BEACON() != address(0));
    }

    function test_immutable_registry() public view {
        assertEq(address(vaultFactory.REGISTRY()), address(registry));
    }

    function test_authorizeUpgrade_onlyOwner() public {
        assertEq(vaultFactory.owner(), owner);

        address mockImpl = address(new SLAYVaultFactoryV2(address(0), registry));

        address sender = vm.randomAddress();
        vm.prank(sender);
        // Call to upgradeToAndCall and expect revert
        (bool success, bytes memory returnData) = address(vaultFactory).call(
            abi.encodeWithSelector(bytes4(keccak256("upgradeToAndCall(address,bytes)")), mockImpl, "")
        );
        assertFalse(success, "Expected upgradeToAndCall to fail");
        assertEq(
            returnData, abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(sender))
        );
    }
}
