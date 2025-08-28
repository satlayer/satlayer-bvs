// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {IPyth} from "@pythnetwork/pyth-sdk-solidity/IPyth.sol";
import {MockPyth} from "@pythnetwork/pyth-sdk-solidity/MockPyth.sol";
import {SLAYOracle} from "../../src/extension/SLAYOracle.sol";
import {ISLAYOracle} from "../../src/extension/interface/ISLAYOracle.sol";
import {SLAYBase} from "../../src/SLAYBase.sol";
import {ISLAYVaultV2} from "../../src/interface/ISLAYVaultV2.sol";
import {MockERC20} from "../MockERC20.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "../TestSuiteV2.sol";

contract SLAYOracleTest is Test, TestSuiteV2 {
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
            address(slayOracle),
            address(new SLAYOracle()),
            abi.encodeCall(SLAYOracle.initialize2, (address(mockPyth), address(router)))
        );
        vm.stopPrank();

        // operator creates a vault
        vm.prank(operator);
        address vaultI = address(vaultFactory.create(underlying));
        vault = ISLAYVaultV2(vaultI);

        // whitelist the vault in router
        vm.prank(owner);
        router.setVaultWhitelist(vaultI, true);

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
        vm.expectEmit();
        emit ISLAYOracle.PriceIdSet(address(underlying), newPriceId);
        slayOracle.setPriceId(address(underlying), newPriceId);

        bytes32 fetchedPriceId = slayOracle.getPriceId(address(underlying));
        assertEq(fetchedPriceId, newPriceId, "Fetched price ID does not match the new one");
    }

    function test_GetPrice() public {
        // call with priceID
        uint256 price = slayOracle.getPrice(priceID);
        // The expected price is $100k in minor units (18 decimals)
        uint256 expectedPrice = 100_000 * 1e18;
        assertEq(price, expectedPrice, "Fetched price does not match the expected one");

        // call with asset
        uint256 priceWithAsset = slayOracle.getPrice(address(underlying));
        assertEq(priceWithAsset, expectedPrice, "Fetched price with asset does not match the expected one");
    }

    function test_revert_GetPrice_NotSet() public {
        // operator create new asset without setting priceID
        MockERC20 underlying2 = new MockERC20("MockWBTC2", "WBTC2", 18);
        vm.expectRevert(abi.encodeWithSelector(ISLAYOracle.PriceIdNotSet.selector, underlying2));
        slayOracle.getPrice(address(underlying2));
    }

    function test_GetOperatorAUM() public {
        address operator2 = makeAddr("Operator2");
        address staker = makeAddr("Staker");

        // register operator
        vm.prank(operator2);
        registry.registerAsOperator("www.operator2.com", "operator2");

        address[] memory vaults;
        vaults = new address[](5);

        // create multiple vault for operator2
        for (uint256 i = 0; i < 5; i++) {
            vm.startPrank(operator2);
            address vaultI = address(vaultFactory.create(underlying));
            vaults[i] = vaultI;
            vm.stopPrank();

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

        uint256 aum = slayOracle.getOperatorAUM(operator2);
        assertEq(aum, 5_000_000 * 1e18); // 5 vaults * 10 wbtc * 100_000 usd/wbtc
    }

    function test_GetVaultAUM() public {
        address staker = makeAddr("Staker");

        // mint tokens to staker
        underlying.mint(staker, 99 * underlyingMinorUnit);
        vm.startPrank(staker);
        underlying.approve(address(vault), 99 * underlyingMinorUnit);
        vault.deposit(99 * underlyingMinorUnit, staker);
        vm.stopPrank();

        uint256 aum = slayOracle.getVaultAUM(address(vault));
        assertEq(aum, 9_900_000 * 1e18); // 99 wbtc * 100_000 usd/wbtc
    }
}
