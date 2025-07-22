// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVaultV2.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "./TestSuiteV2.sol";
import {IERC20Errors} from "@openzeppelin/contracts/interfaces/draft-IERC6093.sol";
import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

contract SLAYVaultV2Test is Test, TestSuiteV2 {
    MockERC20 public underlying = new MockERC20("Wrapped Bitcoin", "WBTC", 8);
    address public immutable operator = makeAddr("Operator Y");
    SLAYVaultV2 public vault;
    MockERC20 btcToken;

    struct StakerInfo {
        address addr;
        address operatorAddr;
        uint256 depositAmount;
        uint256 redeemShares;
        uint256 assetsToReceive;
        uint256 requestId;
    }

    function setUp() public override {
        TestSuiteV2.setUp();

        vm.startPrank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");
        vault = vaultFactory.create(underlying);
        vm.stopPrank();

        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        vm.prank(operator);
        uint32 withdrawalDelay = 8 days;
        registry.setWithdrawalDelay(withdrawalDelay);

        btcToken = MockERC20(vault.asset());
        uint256 decimals = vault.decimals();

        uint256 numInitialStakers = 800; // Number of stakers to "fatten" the vault
        StakerInfo[] memory initialStakers = new StakerInfo[](numInitialStakers);

        for (uint256 i = 0; i < numInitialStakers; i++) {
            address staker = makeAddr(string.concat("staker", vm.toString(i)));
            uint256 depositAmount = (100 + i) * (10 ** decimals); // Varying deposit amounts

            initialStakers[i].addr = staker;
            initialStakers[i].depositAmount = depositAmount; // Store for potential future use or just for context

            // Mint and approve underlying asset
            btcToken.mint(staker, depositAmount);
            vm.startPrank(staker);
            btcToken.approve(address(vault), type(uint256).max);
            vault.deposit(depositAmount, staker);
            vm.stopPrank();
        }
    }

    function test_gas_deposit_request_redeem() public {
        address firstAccount = makeAddr("firstAccount");
        uint256 mintAmount = 1000 * 10 ** btcToken.decimals();
        btcToken.mint(firstAccount, mintAmount);

        // deposit by firstAccount
        vm.startPrank(firstAccount);
        btcToken.approve(address(vault), type(uint256).max);
        uint256 depositAmount = 100 * 10 ** btcToken.decimals();

        vm.startSnapshotGas("SLAYVaultV2", "deposit()");
        vault.deposit(depositAmount, firstAccount);
        vm.stopSnapshotGas();

        assertEq(btcToken.balanceOf(firstAccount), 900 * 10 ** btcToken.decimals()); // mintAmount - depositAmount

        uint256 sharesToWithdraw = 50 * 10 ** vault.decimals();
        vault.approve(address(vault), type(uint256).max);

        vm.startSnapshotGas("SLAYVaultV2", "requestRedeem()");
        vault.requestRedeem(sharesToWithdraw, firstAccount, firstAccount);
        vm.stopSnapshotGas();

        skip(10 days);

        vm.startSnapshotGas("SLAYVaultV2", "redeem()");
        vault.redeem(sharesToWithdraw, firstAccount, firstAccount);
        vm.stopSnapshotGas();

        vm.stopPrank();
    }

    function test_gas_withdraw() public {
        uint8 vaultDecimal = vault.decimals();
        uint256 vaultMinorUnit = 10 ** vaultDecimal;

        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        address firstAccount = makeAddr("firstAccount");
        uint256 mintAmount = 1000 * underlyingMinorUnit;
        underlying.mint(firstAccount, mintAmount);

        vm.startPrank(firstAccount);
        underlying.approve(address(vault), type(uint256).max);
        uint256 depositAmount = 100 * underlyingMinorUnit;
        vault.deposit(depositAmount, firstAccount);

        assertEq(underlying.balanceOf(firstAccount), 900 * underlyingMinorUnit); // mintAmount - depositAmount

        uint256 sharesToWithdraw = 50 * vaultMinorUnit;
        vault.approve(address(vault), type(uint256).max);
        vault.requestRedeem(sharesToWithdraw, firstAccount, firstAccount);

        skip(8 days);

        uint256 maxAssetToWithdraw = vault.maxWithdraw(firstAccount);

        vm.startSnapshotGas("SLAYVaultV2", "withdraw()");
        vault.withdraw(maxAssetToWithdraw, firstAccount, firstAccount);
        vm.stopSnapshotGas();

        vm.stopPrank();
    }
}
