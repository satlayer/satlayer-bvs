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
    constructor(string memory name_, string memory symbol_) initializer {
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

contract SLAYERC20CheckpointedTest is Test, TestSuiteV2 {
    MockSLAYERC20Checkpointed token;
    address alice;
    address bob;

    function setUp() public override {
        TestSuiteV2.setUp();
        token = new MockSLAYERC20Checkpointed("Checkpointed Token", "CHK");
        alice = makeAddr("Alice");
        bob = makeAddr("Bob");
        // Mint initial balances
        token.mint(alice, 1_000 ether);
        token.mint(bob, 500 ether);
    }

    function test_Metadata() public {
        assertEq(token.name(), "Checkpointed Token");
        assertEq(token.symbol(), "CHK");
        assertEq(token.decimals(), 18);
    }

    function test_MintUpdatesBalanceAndTotalSupply() public {
        address carol = makeAddr("Carol");
        uint256 beforeSupply = token.totalSupply();
        token.mint(carol, 123 ether);
        assertEq(token.balanceOf(carol), 123 ether);
        assertEq(token.totalSupply(), beforeSupply + 123 ether);
    }

    function test_BurnUpdatesBalanceAndTotalSupply() public {
        uint256 beforeSupply = token.totalSupply();
        vm.prank(alice);
        // burn from alice via external burner
        token.burn(alice, 100 ether);
        assertEq(token.balanceOf(alice), 900 ether);
        assertEq(token.totalSupply(), beforeSupply - 100 ether);
    }

    function test_Transfer() public {
        vm.prank(alice);
        bool ok = token.transfer(bob, 200 ether);
        assertTrue(ok);
        assertEq(token.balanceOf(alice), 800 ether);
        assertEq(token.balanceOf(bob), 700 ether);
    }

    function test_Revert_Transfer_InsufficientBalance() public {
        address dave = makeAddr("Dave");
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, dave, 0, 1 ether));
        vm.prank(dave);
        token.transfer(alice, 1 ether);
    }

    function test_Revert_Transfer_ToZeroAddress() public {
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InvalidReceiver.selector, address(0)));
        vm.prank(alice);
        token.transfer(address(0), 1 ether);
    }

    function test_ApproveAndAllowance() public {
        vm.prank(alice);
        bool ok = token.approve(bob, 250 ether);
        assertTrue(ok);
        assertEq(token.allowance(alice, bob), 250 ether);

        // update allowance
        vm.prank(alice);
        token.approve(bob, 100 ether);
        assertEq(token.allowance(alice, bob), 100 ether);
    }

    function test_Revert_Approve_ZeroSpender() public {
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InvalidSpender.selector, address(0)));
        vm.prank(alice);
        token.approve(address(0), 1);
    }

    function test_TransferFrom_DecrementsAllowance() public {
        vm.startPrank(alice);
        token.approve(bob, 300 ether);
        vm.stopPrank();

        // Bob spends 120 ether
        vm.prank(bob);
        token.transferFrom(alice, bob, 120 ether);

        assertEq(token.balanceOf(alice), 880 ether);
        assertEq(token.balanceOf(bob), 620 ether);
        assertEq(token.allowance(alice, bob), 180 ether);
    }

    function test_TransferFrom_MaxAllowance_NotDecremented_AndNoApprovalEvent() public {
        // Alice sets infinite approval to Bob
        vm.prank(alice);
        token.approve(bob, type(uint224).max);
        assertEq(token.allowance(alice, bob), type(uint224).max);

        // Record logs to ensure no Approval event is emitted during transferFrom spending
        vm.recordLogs();
        vm.prank(bob);
        token.transferFrom(alice, bob, 77 ether);
        VmSafe.Log[] memory logs = vm.getRecordedLogs();

        // Verify balances and infinite allowance unchanged
        assertEq(token.balanceOf(alice), 923 ether);
        assertEq(token.balanceOf(bob), 577 ether);
        assertEq(token.allowance(alice, bob), type(uint224).max);

        // Ensure no Approval event during transferFrom
        // Approval topic = keccak256("Approval(address,address,uint256)")
        bytes32 approvalTopic = keccak256("Approval(address,address,uint256)");
        for (uint256 i = 0; i < logs.length; i++) {
            assertTrue(logs[i].topics.length == 0 || logs[i].topics[0] != approvalTopic, "unexpected Approval emitted");
        }
    }

    function test_Revert_TransferFrom_InsufficientAllowance() public {
        vm.prank(alice);
        token.approve(bob, 10 ether);
        vm.prank(bob);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientAllowance.selector, bob, 10 ether, 11 ether)
        );
        token.transferFrom(alice, bob, 11 ether);
    }

    function test_Revert_MintToZeroAddress() public {
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InvalidReceiver.selector, address(0)));
        token.mint(address(0), 1);
    }

    function test_Revert_BurnFromZeroAddress() public {
        vm.expectRevert(abi.encodeWithSelector(IERC20Errors.ERC20InvalidSender.selector, address(0)));
        token.burn(address(0), 1);
    }

    function test_CreateDerivative() public {
        // move block forward to create some checkpoints
        vm.roll(10);

        // create derivative token from checkpointed token
        SLAYERC20Derivative derivative = token.createDerivative();
        assertEq(derivative.name(), "Derivative Checkpointed Token");
        assertEq(derivative.symbol(), "dCHK");
        assertEq(derivative.decimals(), 18);

        // assert the derivative has the same total supply
        assertEq(derivative.totalSupply(), token.totalSupply());

        // assert that alice and bob have the same balance in the derivative
        assertEq(derivative.balanceOf(alice), token.balanceOf(alice));
        assertEq(derivative.balanceOf(bob), token.balanceOf(bob));

        // alice transfer derivative token to bob
        vm.prank(alice);
        derivative.transfer(bob, 100 ether);
        assertEq(derivative.balanceOf(alice), 900 ether);
        assertEq(derivative.balanceOf(bob), 600 ether);

        // assert that the checkpointed token balances are unchanged
        assertEq(token.balanceOf(alice), 1000 ether);
        assertEq(token.balanceOf(bob), 500 ether);

        // assert the derivative still have the same total supply
        assertEq(derivative.totalSupply(), token.totalSupply());
    }

    function test_totalSupply() public {
        assertEq(token.totalSupply(), 1500 ether);

        vm.roll(block.number + 1); // move to next block to record checkpoint

        // mint token to new user
        address carol = makeAddr("Carol");
        token.mint(carol, 200 ether);

        // total supply should increase
        assertEq(token.totalSupply(), 1700 ether);

        vm.roll(block.number + 1); // move to next block to record checkpoint

        // burn some token from bob
        vm.prank(bob);
        token.burn(bob, 100 ether);

        // total supply should decrease
        assertEq(token.totalSupply(), 1600 ether);

        vm.roll(block.number + 1); // move to next block to record checkpoint

        // transfer should not affect total supply
        vm.prank(alice);
        token.transfer(bob, 50 ether);
        assertEq(token.totalSupply(), 1600 ether);

        // burn all tokens from carol
        vm.prank(carol);
        token.burn(carol, 200 ether);
        assertEq(token.totalSupply(), 1400 ether);
    }
}
