// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYVault.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYVaultTest is Test, TestSuite {
    MockERC20 public underlying = new MockERC20("Wrapped Bitcoin", "WBTC", 8);
    address public immutable operator = makeAddr("Operator Y");

    function setUp() public override {
        TestSuite.setUp();

        vm.startPrank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");
        vm.stopPrank();
    }

    function test_whitelisted() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        assertTrue(router.whitelisted(address(vault)));
        vm.stopPrank();

        address account = address(this);
        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);
        vault.deposit(100 * 10 ** vault.decimals(), account);
        assertEq(vault.balanceOf(account), 100 * 10 ** vault.decimals());
    }

    function test_notWhitelisted() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        address account = address(this);

        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);

        vm.expectRevert(SLAYVault.ExpectedWhitelisted.selector);
        vault.deposit(100, account);
    }

    function test_paused() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        router.pause();
        vm.stopPrank();

        address account = address(this);

        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);

        vm.expectRevert(SLAYVault.EnforcedPause.selector);
        vault.deposit(100, account);
    }

    function test_deposit() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        vm.stopPrank();

        MockERC20 btcToken = MockERC20(vault.asset());
        address firstAccount = makeAddr("firstAccount");
        uint256 mintAmount = 1000 * 10 ** btcToken.decimals();
        btcToken.mint(firstAccount, mintAmount);
        vm.startPrank(firstAccount);
        btcToken.approve(address(vault), type(uint256).max);

        uint256 depositAmount = 100 * 10 ** btcToken.decimals();
        vault.deposit(depositAmount, firstAccount);
        vm.stopPrank();
        // assert that vault balance is increased by the deposit amount
        assertEq(btcToken.balanceOf(address(vault)), 100 * 10 ** btcToken.decimals()); // depositAmount
        // assert that the account received the correct amount of receipt tokens
        assertEq(vault.balanceOf(firstAccount), 100 * 10 ** vault.decimals()); // depositAmount
        // assert that the account's btcToken balance is decreased by the deposit amount
        assertEq(btcToken.balanceOf(firstAccount), 900 * 10 ** btcToken.decimals()); // mintAmount - depositAmount

        // deposit by another account
        address anotherAccount = makeAddr("anotherAccount");
        btcToken.mint(anotherAccount, mintAmount);
        vm.startPrank(anotherAccount);
        btcToken.approve(address(vault), type(uint256).max);

        uint256 anotherDepositAmount = 200 * 10 ** btcToken.decimals();
        vault.deposit(anotherDepositAmount, anotherAccount);
        vm.stopPrank();
        // assert that vault balance is increased by the deposit amount
        assertEq(btcToken.balanceOf(address(vault)), 300 * 10 ** btcToken.decimals()); // depositAmount + anotherDepositAmount
        // assert that the anotherAccount received the correct amount of receipt tokens
        assertEq(vault.balanceOf(anotherAccount), 200 * 10 ** vault.decimals()); // anotherDepositAmount
        // assert that the another account's btcToken balance is decreased by the deposit amount
        assertEq(btcToken.balanceOf(anotherAccount), 800 * 10 ** btcToken.decimals()); // mintAmount - anotherDepositAmount

        // second deposit by the first account
        uint256 secondDepositAmount = 50 * 10 ** btcToken.decimals();
        vm.startPrank(firstAccount);
        vault.deposit(secondDepositAmount, firstAccount);
        vm.stopPrank();
        // assert that vault balance is increased by the deposit amount
        assertEq(btcToken.balanceOf(address(vault)), 350 * 10 ** btcToken.decimals()); // depositAmount + anotherDepositAmount + secondDepositAmount
        // assert that the account received the correct amount of receipt tokens
        assertEq(vault.balanceOf(firstAccount), 150 * 10 ** vault.decimals()); // depositAmount + secondDepositAmount
        // assert that the account's btcToken balance is decreased by the second deposit amount
        assertEq(btcToken.balanceOf(firstAccount), 850 * 10 ** btcToken.decimals()); // mintAmount - depositAmount - secondDepositAmount
    }

    function testFuzz_deposit(uint256 fuzzAmount) public {
        // Bound the fuzz amount to avoid overflows and unrealistic values
        // Minimum amount is 1, maximum is 1000 * 10^8 (1000 BTC)
        vm.assume(fuzzAmount > 0 && fuzzAmount <= 1000 * 10 ** 8);

        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        vm.stopPrank();

        MockERC20 btcToken = MockERC20(vault.asset());
        address testAccount = makeAddr("testAccount");

        // Mint enough tokens to cover the fuzz amount (add some buffer)
        uint256 mintAmount = fuzzAmount * 2;
        btcToken.mint(testAccount, mintAmount);

        vm.startPrank(testAccount);
        btcToken.approve(address(vault), type(uint256).max);

        // Store balances before deposit
        uint256 accountBalanceBefore = btcToken.balanceOf(testAccount);
        uint256 vaultBalanceBefore = btcToken.balanceOf(address(vault));
        uint256 shareBalanceBefore = vault.balanceOf(testAccount);

        // Perform deposit with fuzzed amount
        vault.deposit(fuzzAmount, testAccount);
        vm.stopPrank();

        // Verify the deposit worked correctly
        // 1. Vault's token balance should increase by the deposit amount
        assertEq(btcToken.balanceOf(address(vault)), vaultBalanceBefore + fuzzAmount);

        // 2. Account should receive the correct amount of shares
        // In this simple case with 1:1 ratio, shares should equal the deposit amount
        assertEq(vault.balanceOf(testAccount), shareBalanceBefore + fuzzAmount);

        // 3. Account's token balance should decrease by the deposit amount
        assertEq(btcToken.balanceOf(testAccount), accountBalanceBefore - fuzzAmount);
    }

    function testFuzz_depositToReceiver(uint256 fuzzAmount) public {
        // Bound the fuzz amount to avoid overflows and unrealistic values
        vm.assume(fuzzAmount > 0 && fuzzAmount <= 1000 * 10 ** 8);

        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        vm.stopPrank();

        MockERC20 btcToken = MockERC20(vault.asset());
        address depositor = makeAddr("depositor");
        address receiver = makeAddr("receiver");

        // Mint tokens to depositor
        uint256 mintAmount = fuzzAmount * 2;
        btcToken.mint(depositor, mintAmount);

        vm.startPrank(depositor);
        btcToken.approve(address(vault), type(uint256).max);

        // Store balances before deposit
        uint256 depositorBalanceBefore = btcToken.balanceOf(depositor);
        uint256 vaultBalanceBefore = btcToken.balanceOf(address(vault));
        uint256 receiverSharesBefore = vault.balanceOf(receiver);

        // Deposit to a different receiver
        vault.deposit(fuzzAmount, receiver);
        vm.stopPrank();

        // Verify the deposit worked correctly
        // 1. Vault's token balance should increase by the deposit amount
        assertEq(btcToken.balanceOf(address(vault)), vaultBalanceBefore + fuzzAmount);

        // 2. Receiver should receive the correct amount of shares
        assertEq(vault.balanceOf(receiver), receiverSharesBefore + fuzzAmount);

        // 3. Depositor's token balance should decrease by the deposit amount
        assertEq(btcToken.balanceOf(depositor), depositorBalanceBefore - fuzzAmount);

        // 4. Depositor should not receive any shares
        assertEq(vault.balanceOf(depositor), 0);
    }

    function testFuzz_depositWithExchangeRate(uint256 initialAssets, uint256 depositAmount) public {
        // Bound the values to avoid overflows and unrealistic values
        // Initial assets between 1 and 1000 * 10^8
        vm.assume(initialAssets > 0 && initialAssets <= 1000 * 10 ** 8);
        // Deposit amount between 1 and 500 * 10^8
        vm.assume(depositAmount > 0 && depositAmount <= 500 * 10 ** 8);

        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        vm.stopPrank();

        MockERC20 btcToken = MockERC20(vault.asset());

        // Create initial state with assets and shares to establish an exchange rate
        address initialDepositor = makeAddr("initialDepositor");
        btcToken.mint(initialDepositor, initialAssets);

        vm.startPrank(initialDepositor);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialAssets, initialDepositor);
        vm.stopPrank();

        uint256 expectedShares = vault.convertToShares(depositAmount);

        // Now test a deposit with the new exchange rate
        address testAccount = makeAddr("testAccount");
        btcToken.mint(testAccount, depositAmount);

        vm.startPrank(testAccount);
        btcToken.approve(address(vault), type(uint256).max);

        // Store balances before deposit
        uint256 accountBalanceBefore = btcToken.balanceOf(testAccount);
        uint256 vaultAssetsBefore = btcToken.balanceOf(address(vault));
        uint256 accountSharesBefore = vault.balanceOf(testAccount);

        // Perform deposit with fuzzed amount
        vault.deposit(depositAmount, testAccount);
        vm.stopPrank();

        // Verify the deposit worked correctly with the exchange rate
        // 1. Vault's asset balance should increase by the deposit amount
        assertEq(btcToken.balanceOf(address(vault)), vaultAssetsBefore + depositAmount);

        // 2. Account should receive the correct amount of shares based on the exchange rate
        assertEq(vault.balanceOf(testAccount), accountSharesBefore + expectedShares);

        // 3. Account's asset balance should decrease by the deposit amount
        assertEq(btcToken.balanceOf(testAccount), accountBalanceBefore - depositAmount);
    }

    function test_redeem() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        vm.stopPrank();

        MockERC20 btcToken = MockERC20(vault.asset());
        address firstAccount = makeAddr("firstAccount");
        uint256 mintAmount = 1000 * 10 ** btcToken.decimals();
        btcToken.mint(firstAccount, mintAmount);

        // deposit by firstAccount
        vm.startPrank(firstAccount);
        btcToken.approve(address(vault), type(uint256).max);
        uint256 depositAmount = 100 * 10 ** btcToken.decimals();
        vault.deposit(depositAmount, firstAccount);

        // assert that the first account btcToken balance is decreased by the deposit amount
        assertEq(btcToken.balanceOf(firstAccount), 900 * 10 ** btcToken.decimals()); // mintAmount - depositAmount

        // request withdraw for first account
        uint256 sharesToWithdraw = 50 * 10 ** vault.decimals();
        vault.approve(address(vault), type(uint256).max);
        vault.requestRedeem(sharesToWithdraw, firstAccount, firstAccount);

        // fast forward to after withdrawal delay
        skip(7 days);

        // execute redeem request after delay
        vault.redeem(sharesToWithdraw, firstAccount, firstAccount);
        vm.stopPrank();

        // assert that vault balance is decreased by the withdraw amount
        assertEq(btcToken.balanceOf(address(vault)), 50 * 10 ** btcToken.decimals()); // depositAmount - sharesToWithdraw
        // assert that the account received the correct amount of btcToken tokens
        assertEq(btcToken.balanceOf(firstAccount), 950 * 10 ** btcToken.decimals()); // mintAmount - depositAmount + sharesToWithdraw
        // assert that the account's receipt tokens are decreased by the withdraw amount
        assertEq(vault.balanceOf(firstAccount), 50 * 10 ** vault.decimals()); // depositAmount - sharesToWithdraw
    }

    function test_withdraw() public {
        vm.prank(operator);
        SLAYVault vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        vm.stopPrank();

        MockERC20 btcToken = MockERC20(vault.asset());
        address firstAccount = makeAddr("firstAccount");
        uint256 mintAmount = 1000 * 10 ** btcToken.decimals();
        btcToken.mint(firstAccount, mintAmount);

        // deposit by firstAccount
        vm.startPrank(firstAccount);
        btcToken.approve(address(vault), type(uint256).max);
        uint256 depositAmount = 100 * 10 ** btcToken.decimals();
        vault.deposit(depositAmount, firstAccount);

        // assert that the first account btcToken balance is decreased by the deposit amount
        assertEq(btcToken.balanceOf(firstAccount), 900 * 10 ** btcToken.decimals()); // mintAmount - depositAmount

        // request withdraw for first account
        uint256 sharesToWithdraw = 50 * 10 ** vault.decimals();
        vault.approve(address(vault), type(uint256).max);
        vault.requestRedeem(sharesToWithdraw, firstAccount, firstAccount);

        // fast forward to after withdrawal delay
        skip(7 days);

        // execute withdrawal request after delay
        uint256 maxAssetToWithdraw = vault.maxWithdraw(firstAccount);
        vault.withdraw(maxAssetToWithdraw, firstAccount, firstAccount);
        vm.stopPrank();

        // assert that vault balance is decreased by the withdraw amount
        assertEq(btcToken.balanceOf(address(vault)), 50 * 10 ** btcToken.decimals()); // depositAmount - sharesToWithdraw
        // assert that the account received the correct amount of btcToken tokens
        assertEq(btcToken.balanceOf(firstAccount), 950 * 10 ** btcToken.decimals()); // mintAmount - depositAmount + sharesToWithdraw
        // assert that the account's receipt tokens are decreased by the withdraw amount
        assertEq(vault.balanceOf(firstAccount), 50 * 10 ** vault.decimals()); // depositAmount - sharesToWithdraw
    }
}
