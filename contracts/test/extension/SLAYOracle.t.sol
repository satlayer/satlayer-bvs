// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {SLAYOracle} from "../../src/extension/SLAYOracle.sol";
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

    function setUp() public override {
        TestSuiteV2.setUp();

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

        // set mapping of asset address to Pyth price ID
        vm.prank(owner);
        slayOracle.setPriceId(address(underlying), priceID);

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
        bytes32 fetchedPriceId = slayOracle.getPriceId(address(underlying));
        assertEq(fetchedPriceId, priceID, "Fetched price ID does not match the expected one");
    }

    function test_SetPriceId() public {
        bytes32 newPriceId = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        vm.prank(owner);
        slayOracle.setPriceId(address(underlying), newPriceId);

        bytes32 fetchedPriceId = slayOracle.getPriceId(address(underlying));
        assertEq(fetchedPriceId, newPriceId, "Fetched price ID does not match the new one");
    }

    function test_revert_SetPriceId_NotOwner() public {
        bytes32 newPriceId = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        slayOracle.setPriceId(address(underlying), newPriceId);
    }

    function test_getPrice() public {
        uint256 price = slayOracle.getPrice(address(underlying));
        // The expected price is $100k in minor units (18 decimals)
        uint256 expectedPrice = 100_000 * 1e18;
        assertEq(price, expectedPrice, "Fetched price does not match the expected one");

        // call with asset
        uint256 priceWithAsset = slayOracle.getPrice(address(underlying));
        assertEq(priceWithAsset, expectedPrice, "Fetched price with asset does not match the expected one");
    }
}
