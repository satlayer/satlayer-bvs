// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {SLAYSDK} from "../../src/addon/SLAYSDK.sol";
import {TestSuiteV2} from "../TestSuiteV2.sol";
import {MockPyth} from "@pythnetwork/pyth-sdk-solidity/MockPyth.sol";
import {Test, console} from "forge-std/Test.sol";

contract SLAYSDKTest is Test, TestSuiteV2 {
    SLAYSDK public slayOracle;

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

        // init SLAYOracle with the router address
        slayOracle = new SLAYOracle(address(router), address(mockPyth));

        // set mapping of asset address to Pyth price ID
        slayOracle.setPythPriceId(address(underlying), priceID);

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

    function test_GetOperatorAUM() public {
        address operator = makeAddr("Operator");
        address staker = makeAddr("Staker");

        address[] memory vaults;
        vaults = new address[](5);

        // register operator
        vm.prank(operator);
        registry.registerAsOperator("www.operator.com", "Operator");

        // create multiple vault for operator
        for (uint256 i = 0; i < 5; i++) {
            vm.prank(operator);
            address vaultI = address(vaultFactory.create(underlying));
            vaults[i] = vaultI;

            vm.prank(owner);
            router.setVaultWhitelist(vaultI, true);
        }

        // deposit some tokens into each vault
        for (uint256 i = 0; i < 5; i++) {
            // mint tokens to staker
            underlying.mint(staker, 10 * underlyingMinorUnit);
            vm.startPrank(staker);
            underlying.approve(vaults[i], 10 * underlyingMinorUnit);
            ISLAYVaultV2(vaults[i]).deposit(10 * underlyingMinorUnit, staker);
            vm.stopPrank();
        }

        uint256 aum = slayOracle.getOperatorAUM(operator);
        assertEq(aum, 5_000_000 * 1e18); // 5 vaults * 10 wbtc * 100_000 usd/wbtc
    }
}
