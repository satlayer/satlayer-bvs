// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {VmSafe} from "forge-std/Vm.sol";
import {TestSuiteV2} from "../TestSuiteV2.sol";
import {SLAYERC20Checkpointed} from "../../src/extension/SLAYERC20Checkpointed.sol";
import {SLAYERC20Derivative} from "../../src/extension/SLAYERC20Derivative.sol";
import {IERC20Errors} from "@openzeppelin/contracts/interfaces/draft-IERC6093.sol";

import {console} from "forge-std/console.sol";

contract MockSLAYERC20Checkpointed is SLAYERC20Checkpointed {
    constructor(string memory name_, string memory symbol_) {
        __ERC20_init(name_, symbol_);
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external {
        _burn(from, amount);
    }

    function createDerivative() external returns (SLAYERC20Derivative) {
        return _createDerivative(name(), symbol());
    }
}

contract SLAYERC20DerivativeTest is Test, TestSuiteV2 {
    MockSLAYERC20Checkpointed base;
    SLAYERC20Derivative derivative;

    address alice;
    address bob;
    address carol;

    function setUp() public override {
        TestSuiteV2.setUp();
        base = new MockSLAYERC20Checkpointed("Checkpointed Token", "CHK");
        alice = makeAddr("Alice");
        bob = makeAddr("Bob");
        carol = makeAddr("Carol");

        // Mint initial balances on checkpointed token
        base.mint(alice, 1_000 ether);
        base.mint(bob, 500 ether);

        // Move a few blocks to ensure non-zero block for snapshot mechanics
        vm.roll(block.number + 5);

        // Create derivative at snapshot = current block
        derivative = base.createDerivative();

        // NOTE: this is crucial to ensure that derivative only allowed to start at a later block than the snapshot
        vm.roll(block.number + 1); // ensure derivative is at a later block than base snapshot
    }

    function test_Metadata() public {
        assertEq(derivative.name(), "Derivative Checkpointed Token");
        assertEq(derivative.symbol(), "dCHK");
        assertEq(derivative.decimals(), 18);
    }

    function test_TotalSupplyInitializedFromSnapshot() public {
        assertEq(derivative.totalSupply(), base.totalSupply());

        // Mutate base after derivative creation; derivative supply must not change
        base.mint(carol, 123 ether);

        vm.roll(block.number + 10); // ensure block change

        assertEq(base.totalSupply(), 1_623 ether); // 1000 + 500 + 123
        assertEq(derivative.totalSupply(), 1_500 ether); // fixed at snapshot
    }

    function test_InitialBalancesMatchSnapshotAndIndependentAfter() public {
        // Initial balances match snapshot
        assertEq(derivative.balanceOf(alice), base.balanceOf(alice));
        assertEq(derivative.balanceOf(bob), base.balanceOf(bob));
        assertEq(derivative.balanceOf(carol), 0); // carol had no balance at snapshot

        // Change base after derivative is created; derivative balances must remain snapshot-based until updated by derivative txs
        base.mint(alice, 200 ether);

        vm.roll(block.number + 10); // ensure block change

        assertEq(base.balanceOf(alice), 1_200 ether);
        // derivative balance remains at snapshot value because it derives from snapshot only
        assertEq(derivative.balanceOf(alice), 1_000 ether);
    }

    function test_Transfer_Succeeds_And_UpdatesBalances() public {
        vm.prank(alice);
        derivative.transfer(bob, 200 ether);
        assertEq(derivative.balanceOf(alice), 800 ether);
        assertEq(derivative.balanceOf(bob), 700 ether);

        // Underlying base token remains unaffected
        assertEq(base.balanceOf(alice), 1_000 ether);
        assertEq(base.balanceOf(bob), 500 ether);
    }

    function test_Transfer_Emits_Transfer_Event() public {
        vm.expectEmit(true, true, false, true, address(derivative));
        emit Transfer(alice, bob, 50 ether);
        vm.prank(alice);
        derivative.transfer(bob, 50 ether);
    }

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    function test_SelfTransfer_NoBalanceChange_ButEvent() public {
        // Record logs to verify event
        vm.recordLogs();
        vm.prank(alice);
        derivative.transfer(alice, 100 ether);

        // Balance unchanged
        assertEq(derivative.balanceOf(alice), 1_000 ether);

        // Ensure one Transfer event with from==to==alice
        VmSafe.Log[] memory logs = vm.getRecordedLogs();
        bytes32 transferTopic = keccak256("Transfer(address,address,uint256)");
        bool found;
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics.length > 0 && logs[i].topics[0] == transferTopic) {
                // topic1 = from, topic2 = to
                assertEq(address(uint160(uint256(logs[i].topics[1]))), alice);
                assertEq(address(uint160(uint256(logs[i].topics[2]))), alice);
                found = true;
            }
        }
        assertTrue(found, "Transfer event not found for self-transfer");
    }

    function test_Revert_Transfer_ToZeroAddress() public {
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InvalidReceiver.selector, address(0)));
        vm.prank(alice);
        derivative.transfer(address(0), 1 ether);
    }

    function test_Revert_Transfer_InsufficientBalance_OnFreshAccount() public {
        address dave = makeAddr("Dave");
        // Dave had zero at snapshot; should revert
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, dave, 0, 1 ether));
        vm.prank(dave);
        derivative.transfer(alice, 1 ether);
    }

    function test_Approve_And_Allowance_Update() public {
        vm.prank(alice);
        bool ok = derivative.approve(bob, 250 ether);
        assertTrue(ok);
        assertEq(derivative.allowance(alice, bob), 250 ether);

        vm.prank(alice);
        derivative.approve(bob, 100 ether);
        assertEq(derivative.allowance(alice, bob), 100 ether);
    }

    function test_Revert_Approve_ZeroSpender() public {
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InvalidSpender.selector, address(0)));
        vm.prank(alice);
        derivative.approve(address(0), 1);
    }

    function test_TransferFrom_DecrementsAllowance() public {
        vm.startPrank(alice);
        derivative.approve(bob, 300 ether);
        vm.stopPrank();

        vm.prank(bob);
        derivative.transferFrom(alice, bob, 120 ether);

        assertEq(derivative.balanceOf(alice), 880 ether);
        assertEq(derivative.balanceOf(bob), 620 ether);
        assertEq(derivative.allowance(alice, bob), 180 ether);
    }

    function test_TransferFrom_MaxAllowance_NotDecremented_AndNoApprovalEvent() public {
        // Alice sets infinite approval to Bob
        vm.prank(alice);
        derivative.approve(bob, type(uint256).max);
        assertEq(derivative.allowance(alice, bob), type(uint256).max);

        // Record logs to ensure no Approval event is emitted during transferFrom spending
        vm.recordLogs();
        vm.prank(bob);
        derivative.transferFrom(alice, bob, 77 ether);
        VmSafe.Log[] memory logs = vm.getRecordedLogs();

        // Verify balances and infinite allowance unchanged
        assertEq(derivative.balanceOf(alice), 923 ether);
        assertEq(derivative.balanceOf(bob), 577 ether);
        assertEq(derivative.allowance(alice, bob), type(uint256).max);

        // Ensure no Approval event during transferFrom
        bytes32 approvalTopic = keccak256("Approval(address,address,uint256)");
        for (uint256 i = 0; i < logs.length; i++) {
            assertTrue(logs[i].topics.length == 0 || logs[i].topics[0] != approvalTopic, "unexpected Approval emitted");
        }
    }

    function test_Revert_TransferFrom_InsufficientAllowance() public {
        vm.prank(alice);
        derivative.approve(bob, 10 ether);
        vm.prank(bob);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientAllowance.selector, bob, 10 ether, 11 ether)
        );
        derivative.transferFrom(alice, bob, 11 ether);
    }

    function test_Revert_TransferFrom_InsufficientBalance() public {
        // Give bob allowance to spend from carol, but carol had 0 at snapshot
        vm.prank(carol);
        derivative.approve(bob, 1 ether);
        vm.prank(bob);
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, carol, 0, 1 ether));
        derivative.transferFrom(carol, bob, 1 ether);
    }

    function test_Transfer_ToNewAccount() public {
        // Ensure Bob hasn't interacted before. First transfer should derive both sides correctly.
        vm.prank(alice);
        derivative.transfer(bob, 1 ether);

        // Balances reflect snapshot-derived starting points +/- transfer
        assertEq(derivative.balanceOf(alice), 999 ether);
        assertEq(derivative.balanceOf(bob), 501 ether);

        // Transfer to a fresh account (carol) who had 0 at snapshot; should work and set her derived balance
        vm.prank(bob);
        derivative.transfer(carol, 1 ether);
        assertEq(derivative.balanceOf(bob), 500 ether);
        assertEq(derivative.balanceOf(carol), 1 ether);
    }

    function test_BaseTokenUnchangedByDerivativeOperations() public {
        uint256 baseAlice = base.balanceOf(alice);
        uint256 baseBob = base.balanceOf(bob);

        vm.prank(alice);
        derivative.transfer(bob, 33 ether);

        assertEq(base.balanceOf(alice), baseAlice);
        assertEq(base.balanceOf(bob), baseBob);
    }
}
