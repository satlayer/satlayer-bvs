// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {SLAYOracle} from "../../src/extension/SLAYOracle.sol";
import {ISLAYOracle} from "../../src/extension/interface/ISLAYOracle.sol";
import {SLAYBase} from "../../src/SLAYBase.sol";
import {IPyth} from "@pythnetwork/pyth-sdk-solidity/IPyth.sol";
import {ISLAYVaultV2} from "../../src/interface/ISLAYVaultV2.sol";
import {MockERC20} from "../MockERC20.sol";
import {MockPyth} from "@pythnetwork/pyth-sdk-solidity/MockPyth.sol";
import {SLAYSDK} from "../../src/extension/SLAYSDK.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "../TestSuiteV2.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

contract SLAYOracleTest is Test, TestSuiteV2 {
    SLAYSDK public slaySDK;
    SLAYOracle public slayOracle;

    MockPyth public mockPyth;

    bytes32 public priceID = 0xc9d8b075a5c69303365ae23633d4e085199bf5c520a3b90fed1322a0342ffc33;

    MockERC20 public underlying = new MockERC20("MockWBTC", "WBTC", 18);
    uint8 public underlyingDecimal = underlying.decimals();
    uint256 public underlyingMinorUnit = 10 ** underlyingDecimal;

    address public operator;
    ISLAYVaultV2 public vault;

    function setUp() public override {
        TestSuiteV2.setUp();
        operator = makeAddr("Operator");
        // register operator
        vm.prank(operator);
        registry.registerAsOperator("www.operator.com", "operator");

        // Creating a mock of Pyth contract with 60 seconds validTimePeriod (for staleness)
        // and 1 wei fee for updating the price.
        mockPyth = new MockPyth(60, 1);

        // base init
        bytes memory baseInit = abi.encodeCall(SLAYBase.initialize, (owner));
        // init SLAYOracle
        slayOracle = SLAYOracle(UnsafeUpgrades.deployUUPSProxy(baseImpl, baseInit));

        // upgrade
        vm.startPrank(owner);
        UnsafeUpgrades.upgradeProxy(
            address(slayOracle), address(new SLAYOracle()), abi.encodeCall(SLAYOracle.initialize2, (address(mockPyth)))
        );
        vm.stopPrank();

        // operator creates a vault
        vm.prank(operator);
        address vaultI = address(vaultFactory.create(underlying));
        vault = ISLAYVaultV2(vaultI);

        // set mapping of asset address to Pyth price ID
        vm.prank(operator);
        slayOracle.setPriceId(address(vault), priceID);

        // update pyth with mock data
        bytes[] memory updateData = new bytes[](1);
        // This is a dummy update data for WBTC. It shows the price as $100k +- $100 (with -8 exponent).
        updateData[0] = mockPyth.createPriceFeedUpdateData(
            priceID,
            100_000 * 1e8, // price in minor units (10^8 for 8 decimals)
            100 * 1e8,
            -8,
            100_000 * 1e8,
            100 * 1e8,
            uint64(block.timestamp),
            uint64(block.timestamp)
        );

        // Make sure the contract has enough funds to update the pyth feeds
        uint256 value = mockPyth.getUpdateFee(updateData);
        vm.deal(address(this), value);

        IPyth pyth = IPyth(address(mockPyth));
        // update the pyth feeds with the mock data
        pyth.updatePriceFeeds{value: value}(updateData);
    }

    function test_GetPriceId() public {
        bytes32 fetchedPriceId = slayOracle.getPriceId(address(vault));
        assertEq(fetchedPriceId, priceID, "Fetched price ID does not match the expected one");
    }

    function test_SetPriceId() public {
        bytes32 newPriceId = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        vm.prank(operator);
        vm.expectEmit();
        emit ISLAYOracle.PriceIdSet(address(vault), newPriceId);
        slayOracle.setPriceId(address(vault), newPriceId);

        bytes32 fetchedPriceId = slayOracle.getPriceId(address(vault));
        assertEq(fetchedPriceId, newPriceId, "Fetched price ID does not match the new one");
    }

    function test_revert_SetPriceId_NotDelegated() public {
        bytes32 newPriceId = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        vm.expectRevert("Only vault's delegated operator can set price ID");
        slayOracle.setPriceId(address(vault), newPriceId);
    }

    function test_getPrice() public {
        // call with priceID
        uint256 price = slayOracle.getPrice(priceID);
        // The expected price is $100k in minor units (18 decimals)
        uint256 expectedPrice = 100_000 * 1e18;
        assertEq(price, expectedPrice, "Fetched price does not match the expected one");

        // call with vault
        uint256 priceWithAsset = slayOracle.getPrice(address(vault));
        assertEq(priceWithAsset, expectedPrice, "Fetched price with asset does not match the expected one");
    }
}
