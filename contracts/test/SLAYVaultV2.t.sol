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

    function setUp() public override {
        TestSuiteV2.setUp();

        vm.startPrank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");
        vm.stopPrank();
    }

    function test_erc165() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);

        assertTrue(vault.supportsInterface(type(IERC20).interfaceId));
        assertTrue(vault.supportsInterface(type(IERC20Metadata).interfaceId));
        assertTrue(vault.supportsInterface(type(IERC4626).interfaceId));
        assertTrue(vault.supportsInterface(type(IERC7540Redeem).interfaceId));
        assertTrue(vault.supportsInterface(type(IERC7540Operator).interfaceId));
        assertTrue(vault.supportsInterface(type(ISLAYVaultV2).interfaceId));
        // super
        assertTrue(vault.supportsInterface(type(IERC165).interfaceId));
    }

    function test_whitelisted() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);
        assertFalse(vault.isWhitelisted());

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        assertTrue(router.isVaultWhitelisted(address(vault)));
        assertTrue(vault.isWhitelisted());
        vm.stopPrank();

        address account = address(this);
        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);
        vault.deposit(100 * 10 ** vault.decimals(), account);
        assertEq(vault.balanceOf(account), 100 * 10 ** vault.decimals());
    }

    function test_notWhitelisted() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);

        address account = address(this);

        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);

        vm.expectRevert(ISLAYVaultV2.ExpectedWhitelisted.selector);
        vault.deposit(100, account);
    }

    function test_paused() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        router.pause();
        vm.stopPrank();

        address account = address(this);

        underlying.mint(account, 1000 * 10 ** vault.decimals());
        underlying.approve(address(vault), type(uint256).max);

        vm.expectRevert(ISLAYVaultV2.EnforcedPause.selector);
        vault.deposit(100, account);
    }

    function test_deposit() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);

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
        SLAYVaultV2 vault = vaultFactory.create(underlying);

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
        SLAYVaultV2 vault = vaultFactory.create(underlying);

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
        SLAYVaultV2 vault = vaultFactory.create(underlying);

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
        SLAYVaultV2 vault = vaultFactory.create(underlying);

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
        SLAYVaultV2 vault = vaultFactory.create(underlying);
        uint8 vaultDecimal = vault.decimals();
        uint256 vaultMinorUnit = 10 ** vaultDecimal;

        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        address firstAccount = makeAddr("firstAccount");
        uint256 mintAmount = 1000 * underlyingMinorUnit;
        underlying.mint(firstAccount, mintAmount);

        // deposit by firstAccount
        vm.startPrank(firstAccount);
        underlying.approve(address(vault), type(uint256).max);
        uint256 depositAmount = 100 * underlyingMinorUnit;
        vault.deposit(depositAmount, firstAccount);

        // assert that the first account underlying balance is decreased by the deposit amount
        assertEq(underlying.balanceOf(firstAccount), 900 * underlyingMinorUnit); // mintAmount - depositAmount

        // request withdraw for first account
        uint256 sharesToWithdraw = 50 * vaultMinorUnit;
        vault.approve(address(vault), type(uint256).max);
        vault.requestRedeem(sharesToWithdraw, firstAccount, firstAccount);

        // fast forward to after withdrawal delay
        skip(7 days);

        // execute withdrawal request after delay
        uint256 maxAssetToWithdraw = vault.maxWithdraw(firstAccount);
        vault.withdraw(maxAssetToWithdraw, firstAccount, firstAccount);
        vm.stopPrank();

        // assert that vault balance is decreased by the withdraw amount
        assertEq(underlying.balanceOf(address(vault)), 50 * underlyingMinorUnit); // depositAmount - sharesToWithdraw
        // assert that the account received the correct amount of btcToken tokens
        assertEq(underlying.balanceOf(firstAccount), 950 * underlyingMinorUnit); // mintAmount - depositAmount + sharesToWithdraw
        // assert that the account's receipt tokens are decreased by the withdraw amount
        assertEq(vault.balanceOf(firstAccount), 50 * vaultMinorUnit); // depositAmount - sharesToWithdraw
    }

    /**
     * Test lifecycle of the vault for a more complex case:
     *  - Staker1 deposits 100 WBTC into the vault
     *  - Staker2 deposits 1 WBTC into the vault
     *  - Staker1 requests to withdraw 100 WBTC
     *  - Staker3 deposits 5 WBTC into the vault
     *  - Staker4 deposits 1 WBTC for Staker2 into the vault
     *  - Staker2 requests to withdraw 2 WBTC
     *  - Staker1 withdraws 100 WBTC
     *  - Staker2 redeems 2 WBTC
     * Because there is no donation or slashing, the exchange rate should remain 1:1 throughout the process.
     */
    function test_lifecycle_deposit_withdrawal() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);
        uint8 vaultDecimal = vault.decimals();
        uint256 vaultMinorUnit = 10 ** vaultDecimal;

        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        address staker1 = makeAddr("staker1");
        address staker2 = makeAddr("staker2");
        address staker3 = makeAddr("staker3");
        address staker4 = makeAddr("staker4");

        // fund staker1 with 100 WBTC
        underlying.mint(staker1, 100 * underlyingMinorUnit);
        // fund staker2 with 1 WBTC
        underlying.mint(staker2, 1 * underlyingMinorUnit);
        // fund staker3 with 5 WBTC
        underlying.mint(staker3, 5 * underlyingMinorUnit);
        // fund staker4 with 1 WBTC
        underlying.mint(staker4, 1 * underlyingMinorUnit);

        // staker1 deposits 100 WBTC
        vm.startPrank(staker1);
        underlying.approve(address(vault), 100 * underlyingMinorUnit);
        vault.deposit(100 * underlyingMinorUnit, staker1);
        vm.stopPrank();
        // assert that staker1 has 100 receipt tokens
        assertEq(vault.balanceOf(staker1), 100 * vaultMinorUnit);
        // assert that vault balance is 100 WBTC
        assertEq(vault.totalAssets(), 100 * underlyingMinorUnit);

        // staker2 deposits 1 WBTC
        vm.startPrank(staker2);
        underlying.approve(address(vault), 1 * underlyingMinorUnit);
        vault.deposit(1 * underlyingMinorUnit, staker2);
        vm.stopPrank();
        // assert that staker2 has 1 receipt token
        assertEq(vault.balanceOf(staker2), 1 * vaultMinorUnit);
        // assert that vault balance is 101 WBTC (100 + 1)
        assertEq(vault.totalAssets(), 101 * underlyingMinorUnit);

        // staker1 requests to withdraw 100 WBTC
        vm.prank(staker1);
        vault.requestRedeem(100 * vaultMinorUnit, staker1, staker1);
        // assert that staker1's receipt tokens are sent to the vault
        assertEq(vault.balanceOf(staker1), 0);
        assertEq(vault.balanceOf(address(vault)), 100 * vaultMinorUnit);
        // assert that staker1's pending redeem request is 100 WBTC
        assertEq(vault.pendingRedeemRequest(0, staker1), 100 * vaultMinorUnit);

        // staker3 deposits 5 WBTC
        vm.startPrank(staker3);
        underlying.approve(address(vault), 5 * underlyingMinorUnit);
        vault.deposit(5 * underlyingMinorUnit, staker3);
        vm.stopPrank();
        // assert that staker3 has 5 receipt tokens
        assertEq(vault.balanceOf(staker3), 5 * vaultMinorUnit);
        // assert that vault balance is 106 WBTC (101 + 5)
        assertEq(vault.totalAssets(), 106 * underlyingMinorUnit);

        // staker4 deposits 1 WBTC for staker2
        vm.startPrank(staker4);
        underlying.approve(address(vault), 1 * underlyingMinorUnit);
        vault.deposit(1 * underlyingMinorUnit, staker2);
        vm.stopPrank();
        // assert that staker2 has 2 (1+1) receipt token
        assertEq(vault.balanceOf(staker2), 2 * vaultMinorUnit);
        // assert that vault balance is 107 WBTC (106 + 1)
        assertEq(vault.totalAssets(), 107 * underlyingMinorUnit);

        // staker2 requests to withdraw 2 WBTC
        vm.prank(staker2);
        vault.requestRedeem(2 * vaultMinorUnit, staker2, staker2);
        // assert that staker2's receipt tokens are sent to the vault
        assertEq(vault.balanceOf(staker2), 0);
        assertEq(vault.balanceOf(address(vault)), 102 * vaultMinorUnit); // 100 + 2 (pending redeem request)
        // assert that staker2's pending redeem request is 2 WBTC
        assertEq(vault.pendingRedeemRequest(0, staker2), 2 * underlyingMinorUnit);

        // fast forward to after withdrawal delay
        skip(7 days);

        // assert that staker1's redeem request is claimable
        assertEq(vault.claimableRedeemRequest(0, staker1), 100 * underlyingMinorUnit);

        // staker1 withdraws 100 WBTC
        vm.prank(staker1);
        vault.withdraw(100 * underlyingMinorUnit, staker1, staker1);
        // assert that staker1 received 100 WBTC
        assertEq(underlying.balanceOf(staker1), 100 * underlyingMinorUnit);
        // assert that vault's asset is 7 WBTC (107 - 100)
        assertEq(vault.totalAssets(), 7 * underlyingMinorUnit);
        // assert that staker1's redeem request is cleared
        assertEq(vault.pendingRedeemRequest(0, staker1), 0);
        assertEq(vault.claimableRedeemRequest(0, staker1), 0);

        // assert that staker2's redeem request is claimable
        assertEq(vault.claimableRedeemRequest(0, staker2), 2 * underlyingMinorUnit);

        // staker2 redeems 2 WBTC
        vm.prank(staker2);
        vault.redeem(2 * underlyingMinorUnit, staker2, staker2);
        // assert that staker2 received 2 WBTC
        assertEq(underlying.balanceOf(staker2), 2 * underlyingMinorUnit);
        // assert that vault's asset is 5 WBTC (7 - 2)
        assertEq(vault.totalAssets(), 5 * underlyingMinorUnit);
        // assert that staker2's redeem request is cleared
        assertEq(vault.pendingRedeemRequest(0, staker2), 0);
        assertEq(vault.claimableRedeemRequest(0, staker2), 0);
    }

    function test_lifecycle_withdrawal_with_operator() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);
        uint8 vaultDecimal = vault.decimals();
        uint256 vaultMinorUnit = 10 ** vaultDecimal;

        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        address staker1 = makeAddr("staker/1");
        address staker2 = makeAddr("staker/2");
        address operator1 = makeAddr("operator/1");

        // fund staker1 with 100 WBTC
        underlying.mint(staker1, 100 * underlyingMinorUnit);
        // fund staker2 with 5 WBTC
        underlying.mint(staker2, 5 * underlyingMinorUnit);

        // staker1 deposits 100 WBTC
        vm.startPrank(staker1);
        underlying.approve(address(vault), 100 * underlyingMinorUnit);
        vault.deposit(100 * underlyingMinorUnit, staker1);
        vm.stopPrank();
        // assert that staker1 has 100 receipt tokens
        assertEq(vault.balanceOf(staker1), 100 * vaultMinorUnit);
        // assert that vault balance is 100 WBTC
        assertEq(vault.totalAssets(), 100 * underlyingMinorUnit);

        // staker2 deposits 5 WBTC
        vm.startPrank(staker2);
        underlying.approve(address(vault), 5 * underlyingMinorUnit);
        vault.deposit(5 * underlyingMinorUnit, staker2);
        vm.stopPrank();
        // assert that staker2 has 5 receipt token
        assertEq(vault.balanceOf(staker2), 5 * vaultMinorUnit);
        // assert that vault balance is 105 WBTC (100 + 5)
        assertEq(vault.totalAssets(), 105 * underlyingMinorUnit);

        // staker1 approves operator
        vm.prank(staker1);
        vault.setOperator(operator1, true);

        // staker2 approves operator
        vm.prank(staker2);
        vault.setOperator(operator1, true);

        // operator requests to withdraw 100 WBTC for staker1
        vm.prank(operator1);
        vault.requestRedeem(100 * vaultMinorUnit, staker1, staker1);
        // assert that staker1's receipt tokens are sent to the vault
        assertEq(vault.balanceOf(staker1), 0);
        assertEq(vault.balanceOf(address(vault)), 100 * vaultMinorUnit);
        // assert that staker1's pending redeem request is 100 WBTC
        assertEq(vault.pendingRedeemRequest(0, staker1), 100 * vaultMinorUnit);

        // operator requests to withdraw 5 WBTC for staker2
        vm.prank(operator1);
        vault.requestRedeem(5 * vaultMinorUnit, staker2, staker2);
        // assert that staker2's receipt tokens are sent to the vault
        assertEq(vault.balanceOf(staker2), 0);
        assertEq(vault.balanceOf(address(vault)), 105 * vaultMinorUnit); // 100 + 5 (pending redeem request)
        // assert that staker2's pending redeem request is 5 WBTC
        assertEq(vault.pendingRedeemRequest(0, staker2), 5 * vaultMinorUnit);

        // fast forward to after withdrawal delay
        skip(7 days);

        // assert that staker1's redeem request is claimable
        assertEq(vault.claimableRedeemRequest(0, staker1), 100 * underlyingMinorUnit);

        // staker1 withdraws 100 WBTC
        vm.prank(staker1);
        vault.withdraw(100 * underlyingMinorUnit, staker1, staker1);
        // assert that staker1 received 100 WBTC
        assertEq(underlying.balanceOf(staker1), 100 * underlyingMinorUnit);
        // assert that vault's asset is 5 WBTC (105 - 100)
        assertEq(vault.totalAssets(), 5 * underlyingMinorUnit);
        // assert that staker1's redeem request is cleared
        assertEq(vault.pendingRedeemRequest(0, staker1), 0);
        assertEq(vault.claimableRedeemRequest(0, staker1), 0);

        // assert that staker2's redeem request is claimable
        assertEq(vault.claimableRedeemRequest(0, staker2), 5 * underlyingMinorUnit);

        // operator redeems 5 WBTC for staker2
        vm.prank(operator1);
        vault.redeem(5 * underlyingMinorUnit, staker2, staker2);
        // assert that staker2 received 5 WBTC
        assertEq(underlying.balanceOf(staker2), 5 * underlyingMinorUnit);
        // assert that vault's asset is 0 WBTC
        assertEq(vault.totalAssets(), 0 * underlyingMinorUnit);
        // assert that staker2's redeem request is cleared
        assertEq(vault.pendingRedeemRequest(0, staker2), 0);
        assertEq(vault.claimableRedeemRequest(0, staker2), 0);
    }

    /**
     * Demonstrate for the case that controller != owner, owner's operator != controller's operator and owner will not be able to redeem.
     * Hence, controller have all the rights to redeem/withdraw to another address.
     * for requestRedeem:
     *  - only owner can call it
     *  - or owner's operator can call it
     *  - or owner's has approved another address (has allowance) to call it
     * for redeem:
     *  - only controller or controller's operator can call it
     *  - even original staker cannot call redeem (if owner != controller in requestRedeem)
     */
    function test_lifecycle_withdrawal_with_operator_reverts() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);
        uint8 vaultDecimal = vault.decimals();
        uint256 vaultMinorUnit = 10 ** vaultDecimal;

        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        address staker1 = makeAddr("staker/1");
        address staker2 = makeAddr("staker/2");
        address operator1 = makeAddr("operator/1");
        address operator2 = makeAddr("operator/2");
        address randomAddress = makeAddr("randomAddress");

        // fund staker1 with 100 WBTC
        underlying.mint(staker1, 100 * underlyingMinorUnit);
        // fund staker2 with 5 WBTC
        underlying.mint(staker2, 5 * underlyingMinorUnit);

        // staker1 deposits 100 WBTC
        vm.startPrank(staker1);
        underlying.approve(address(vault), 100 * underlyingMinorUnit);
        vault.deposit(100 * underlyingMinorUnit, staker1);
        vm.stopPrank();
        // assert that staker1 has 100 receipt tokens
        assertEq(vault.balanceOf(staker1), 100 * vaultMinorUnit);
        // assert that vault balance is 100 WBTC
        assertEq(vault.totalAssets(), 100 * underlyingMinorUnit);

        // staker2 deposits 5 WBTC
        vm.startPrank(staker2);
        underlying.approve(address(vault), 5 * underlyingMinorUnit);
        vault.deposit(5 * underlyingMinorUnit, staker2);
        vm.stopPrank();
        // assert that staker2 has 5 receipt token
        assertEq(vault.balanceOf(staker2), 5 * vaultMinorUnit);
        // assert that vault balance is 105 WBTC (100 + 5)
        assertEq(vault.totalAssets(), 105 * underlyingMinorUnit);

        // staker2 request redeem 100WBTC for staker1 (revert)
        vm.startPrank(staker2);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientAllowance.selector, staker2, 0, 100 * vaultMinorUnit)
        );
        vault.requestRedeem(100 * vaultMinorUnit, staker1, staker1);
        vm.stopPrank();

        // operator requests to withdraw 100 WBTC for staker1 (revert)
        vm.startPrank(operator1);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientAllowance.selector, operator1, 0, 100 * vaultMinorUnit)
        );
        vault.requestRedeem(100 * vaultMinorUnit, staker1, staker1);
        vm.stopPrank();

        // staker1 approves operator
        vm.prank(staker1);
        vault.setOperator(operator1, true);

        // operator approves operator2
        vm.prank(operator1);
        vault.setOperator(operator2, true);

        // operator2 requests to withdraw 100 WBTC for staker1 (revert)
        vm.startPrank(operator2);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientAllowance.selector, operator2, 0, 100 * vaultMinorUnit)
        );
        vault.requestRedeem(100 * vaultMinorUnit, staker1, staker1);
        vm.stopPrank();

        // staker1 increases allowance for randomAddress
        vm.prank(staker1);
        vault.approve(randomAddress, 100 * vaultMinorUnit);

        // randomAddress requests to withdraw 100 WBTC for staker1 (revert)
        vm.startPrank(randomAddress);
        vm.expectRevert(abi.encodeWithSelector(ISLAYVaultV2.NotControllerOrOperator.selector));
        vault.requestRedeem(100 * vaultMinorUnit, staker1, staker1);
        vm.stopPrank();
    }

    function test_lockSlashing() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);

        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        vm.stopPrank();

        address staker = makeAddr("staker");
        uint256 mintAmount = 1000 * underlyingMinorUnit;
        underlying.mint(staker, mintAmount);

        // deposit by staker
        vm.startPrank(staker);
        underlying.approve(address(vault), type(uint256).max);
        uint256 depositAmount = 100 * underlyingMinorUnit;
        vault.deposit(depositAmount, staker);
        vm.stopPrank();

        // assert that the first account btcToken balance is decreased by the deposit amount
        assertEq(underlying.balanceOf(staker), 900 * underlyingMinorUnit); // mintAmount - depositAmount

        // slash lock called by router
        vm.prank(address(router));
        vm.expectEmit();
        emit ISLAYVaultV2.SlashingLocked(20 * underlyingMinorUnit);
        vault.lockSlashing(20 * underlyingMinorUnit);

        // assert that vault balance is decreased by the slash amount
        assertEq(underlying.balanceOf(address(vault)), 80 * underlyingMinorUnit); // depositAmount - slashAmount

        // assert that the router balance is increased by the slash amount
        assertEq(underlying.balanceOf(address(router)), 20 * underlyingMinorUnit); // slashAmount
    }

    function test_revert_lockSlashing() public {
        vm.prank(operator);
        SLAYVaultV2 vault = vaultFactory.create(underlying);

        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        vm.startPrank(owner);
        router.setVaultWhitelist(address(vault), true);
        vm.stopPrank();

        // Attempt to call lockSlashing from a non-router address
        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(ISLAYVaultV2.NotRouter.selector));
        vault.lockSlashing(20 * underlyingMinorUnit);

        // Attempt to call lockSlashing with zero balance
        vm.prank(address(router));
        vm.expectRevert(
            abi.encodeWithSelector(
                IERC20Errors.ERC20InsufficientBalance.selector, address(vault), 0, 20 * underlyingMinorUnit
            )
        );
        vault.lockSlashing(20 * underlyingMinorUnit);
    }
}
