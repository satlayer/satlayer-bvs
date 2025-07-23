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

    function test_gas_withdraw() public {
        uint8 vaultDecimal = vault.decimals();
        uint256 vaultMinorUnit = 10 ** vaultDecimal;

        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        address firstAccount = makeAddr("xyz");
        uint256 mintAmount = 1000 * underlyingMinorUnit;
        underlying.mint(firstAccount, mintAmount);

        vm.startPrank(firstAccount);
        underlying.approve(address(vault), type(uint256).max);
        uint256 depositAmount = 100 * underlyingMinorUnit;
        vm.startSnapshotGas("SLAYVaultV2", "deposit()");
        vault.deposit(depositAmount, firstAccount);
        vm.stopSnapshotGas();

        assertEq(underlying.balanceOf(firstAccount), 900 * underlyingMinorUnit); // mintAmount - depositAmount

        uint256 sharesToWithdraw = 50 * vaultMinorUnit;
        vault.requestRedeem(sharesToWithdraw, firstAccount, firstAccount);
        vm.stopPrank();

        skip(8 days);

        uint256 maxAssetToWithdraw = vault.maxWithdraw(firstAccount);

        vm.prank(firstAccount);
        vm.startSnapshotGas("SLAYVaultV2", "withdraw()");
        vault.withdraw(maxAssetToWithdraw, firstAccount, firstAccount);
        vm.stopSnapshotGas();

        vm.stopPrank();
    }

    function _assertRequestRedeemSuccess(
        address sender,
        address controller,
        address owner_addr,
        uint256 sharesToRequest,
        uint256 ownerSharesBefore,
        uint256 vaultSharesBefore,
        uint256 totalPendingBefore,
        string memory message
    ) internal {
        assertEq(
            vault.balanceOf(owner_addr),
            ownerSharesBefore - sharesToRequest,
            string.concat(message, ": Shares not transferred from owner.")
        );
        assertEq(
            vault.balanceOf(address(vault)),
            vaultSharesBefore + sharesToRequest,
            string.concat(message, ": Shares not transferred to vault.")
        );
        assertEq(
            vault.pendingRedeemRequest(0, controller),
            sharesToRequest,
            string.concat(message, ": Pending redeem request not set for controller.")
        );
        assertEq(
            vault.getTotalPendingRedemption(),
            totalPendingBefore + sharesToRequest,
            string.concat(message, ": Total pending not updated.")
        );
        console.log("%s: SUCCESS", message);
    }

    // Path 1: Sender: A, Controller: A, Owner: A
    function test_requestRedeem_Path1_SenderControllerOwnerSame() public {
        address a = makeAddr("a-1");
        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        // Setup: A deposits to get shares
        btcToken.mint(a, initialDeposit);
        vm.startPrank(a);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, a);
        vm.stopPrank();

        uint256 a_shares_before = vault.balanceOf(a);
        uint256 vault_shares_before = vault.balanceOf(address(vault));
        uint256 totalPending_before = vault.getTotalPendingRedemption();

        vm.prank(a);
        vm.startSnapshotGas("SLAYVaultV2", "requestRedeem()_sender_a_controller_a_owner_a");
        vault.requestRedeem(sharesToRequest, a, a);
        vm.stopSnapshotGas();

        _assertRequestRedeemSuccess(
            a, a, a, sharesToRequest, a_shares_before, vault_shares_before, totalPending_before, "Path 1 (A-A-A)"
        );

        skip(10 days);
    }

    function test_requestRedeem_Path2_SenderIsOwner_ControllerDifferent_SenderIsControllerOperator() public {
        address sender = makeAddr("a-2");
        address controller = makeAddr("b-2");
        address owner_addr = sender; // Owner is the sender
        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        // Setup 1: Owner (A) deposits to get shares
        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        // Setup 2: Controller (B) approves Sender (A) as its ERC7540 Operator
        vm.startPrank(controller);
        vault.setOperator(sender, true);
        vm.stopPrank();

        uint256 owner_shares_before = vault.balanceOf(owner_addr);
        uint256 vault_shares_before = vault.balanceOf(address(vault));
        uint256 totalPending_before = vault.getTotalPendingRedemption();

        vm.startPrank(sender);
        vm.startSnapshotGas("SLAYVaultV2", "requestRedeem()_sender_a_controller_b_owner_a");
        vault.requestRedeem(sharesToRequest, controller, owner_addr);
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertRequestRedeemSuccess(
            sender,
            controller,
            owner_addr,
            sharesToRequest,
            owner_shares_before,
            vault_shares_before,
            totalPending_before,
            "Path 2 (A-B-A, A is op for B)"
        );
        skip(10 days);
    }

    function test_requestRedeem_Path3_SenderIsController_OwnerDifferent_OwnerApprovedSender() public {
        address sender = makeAddr("a-3");
        address controller = sender; // Controller is the sender
        address owner_addr = makeAddr("b-3");
        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        vm.startPrank(owner_addr);
        vault.approve(sender, sharesToRequest);
        vm.stopPrank();

        uint256 owner_shares_before = vault.balanceOf(owner_addr);
        uint256 vault_shares_before = vault.balanceOf(address(vault));
        uint256 totalPending_before = vault.getTotalPendingRedemption();

        vm.startPrank(sender);
        vm.startSnapshotGas("SLAYVaultV2", "requestRedeem()_sender_a_controller_a_owner_b_owner_approved_sender");
        vault.requestRedeem(sharesToRequest, controller, owner_addr);
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertRequestRedeemSuccess(
            sender,
            controller,
            owner_addr,
            sharesToRequest,
            owner_shares_before,
            vault_shares_before,
            totalPending_before,
            "Path 3 (A-A-B, B approved A)"
        );
        assertEq(vault.allowance(owner_addr, sender), 0, "Path 3: Allowance not consumed correctly.");
        skip(10 days);
    }

    function test_requestRedeem_Path4_SenderIsController_OwnerDifferent_OwnerPermittedSender() public {
        address sender = makeAddr("a-4");
        address controller = sender;
        uint256 owner_private_key = 0xeb902a602b16153984692589c18bba962e70aa11fd33235238e13d5331392866;
        address owner_addr = vm.addr(owner_private_key);

        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        uint256 nonce = vault.nonces(owner_addr);
        uint256 deadline = block.timestamp + 3600;

        bytes32 DOMAIN_SEPARATOR = vault.DOMAIN_SEPARATOR();
        bytes32 structHash = keccak256(
            abi.encode(
                keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"),
                owner_addr,
                sender,
                sharesToRequest,
                nonce,
                deadline
            )
        );
        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", DOMAIN_SEPARATOR, structHash));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(owner_private_key, digest);

        uint256 owner_shares_before = vault.balanceOf(owner_addr);
        uint256 vault_shares_before = vault.balanceOf(address(vault));
        uint256 totalPending_before = vault.getTotalPendingRedemption();

        vm.prank(sender);
        vault.permit(owner_addr, sender, sharesToRequest, deadline, v, r, s); // A submits B's permit signature

        vm.startPrank(sender);
        vm.startSnapshotGas("SLAYVaultV2", "requestRedeem()_sender_a_controller_a_owner_b_permit_signature");
        vault.requestRedeem(sharesToRequest, controller, owner_addr);
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertRequestRedeemSuccess(
            sender,
            controller,
            owner_addr,
            sharesToRequest,
            owner_shares_before,
            vault_shares_before,
            totalPending_before,
            "Path 4 (A-A-B, B permitted A)"
        );
        assertEq(vault.allowance(owner_addr, sender), 0, "Path 4: Allowance not consumed correctly after permit.");
        skip(10 days);
    }

    function test_requestRedeem_Path5_SenderIsController_OwnerDifferent_SenderIsOwnerOperator() public {
        address sender = makeAddr("a-5");
        address controller = sender; // Controller is the sender
        address owner_addr = makeAddr("b-5");
        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        vm.prank(owner_addr);
        vault.setOperator(sender, true);

        uint256 owner_shares_before = vault.balanceOf(owner_addr);
        uint256 vault_shares_before = vault.balanceOf(address(vault));
        uint256 totalPending_before = vault.getTotalPendingRedemption();

        vm.startPrank(sender);
        vm.startSnapshotGas("SLAYVaultV2", "requestRedeem()_sender_a_controller_a_owner_b_via_allowance");
        vault.requestRedeem(sharesToRequest, controller, owner_addr);
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertRequestRedeemSuccess(
            sender,
            controller,
            owner_addr,
            sharesToRequest,
            owner_shares_before,
            vault_shares_before,
            totalPending_before,
            "Path 5 (A-A-B, A is op for B)"
        );
        skip(10 days);
    }

    function test_requestRedeem_Path6_SenderIsOwner_ControllerDifferent_SenderIsControllerOperator() public {
        address sender = makeAddr("a-6"); // This is 'a'
        address controller = makeAddr("c-6"); // This is 'c'
        address owner_addr = sender; // This is 'a'

        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        vm.startPrank(controller);
        vault.setOperator(sender, true); // C approves A as operator
        vm.stopPrank();

        uint256 owner_shares_before = vault.balanceOf(owner_addr);
        uint256 vault_shares_before = vault.balanceOf(address(vault));
        uint256 totalPending_before = vault.getTotalPendingRedemption();

        vm.startPrank(sender); // Sender is 'a'
        vm.startSnapshotGas("SLAYVaultV2", "requestRedeem()_sender_a_controller_c_owner_a");
        vault.requestRedeem(sharesToRequest, controller, owner_addr);
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertRequestRedeemSuccess(
            sender,
            controller,
            owner_addr,
            sharesToRequest,
            owner_shares_before,
            vault_shares_before,
            totalPending_before,
            "Path 6 (A-C-A, A is op for C)"
        );
        skip(10 days);
    }

    function _assertWithdrawSuccess(
        address caller, // msg.sender of the withdraw call
        address receiver,
        address controller,
        uint256 expectedAssetsReceived,
        uint256 expectedSharesBurned, // These shares are burned from the vault itself (they were moved there in requestRedeem)
        uint256 receiverAssetBalanceBefore,
        uint256 vaultSharesBalanceBefore,
        uint256 totalPendingBefore,
        string memory message
    ) internal {
        assertEq(
            underlying.balanceOf(receiver),
            receiverAssetBalanceBefore + expectedAssetsReceived,
            string.concat(message, ": Receiver did not get assets.")
        );
        assertEq(
            vault.balanceOf(address(vault)),
            vaultSharesBalanceBefore - expectedSharesBurned,
            string.concat(message, ": Shares not burned from vault.")
        );
        assertEq(
            vault.getTotalPendingRedemption(),
            totalPendingBefore - expectedSharesBurned,
            string.concat(message, ": Total pending not updated.")
        );
        console.log("%s: SUCCESS", message);
    }

    // Path 1: Sender: A, Controller: A, Owner: A
    function test_withdraw_Path1_SenderControllerOwnerSame() public {
        address a = makeAddr("a-w1");
        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        // Setup: A deposits to get shares
        btcToken.mint(a, initialDeposit);
        vm.startPrank(a);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, a);
        vm.stopPrank();

        // A requests redeem (A is owner, A is controller)
        vm.prank(a);
        // Capture requestId if it's used to identify specific requests.
        // Assuming sequential requests and we can infer the last one by nextRequestId - 1
        uint256 requestId_A_A_A = vault.requestRedeem(sharesToRequest, a, a);

        skip(8 days); // Pass withdrawal delay

        uint256 assetsToWithdraw = vault.maxWithdraw(a); // maxWithdraw for A (who is controller/owner)
        uint256 receiverAssetBalanceBefore = underlying.balanceOf(a);
        uint256 vaultSharesBalanceBefore = vault.balanceOf(address(vault));
        uint256 totalPendingBefore = vault.getTotalPendingRedemption();

        vm.startPrank(a); // Sender is A
        vm.startSnapshotGas("SLAYVaultV2", "withdraw()_sender_a_controller_a_owner_a");
        vault.withdraw(assetsToWithdraw, a, a); // receiver is A, controller is A
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertWithdrawSuccess(
            a,
            a,
            a,
            assetsToWithdraw,
            sharesToRequest,
            receiverAssetBalanceBefore,
            vaultSharesBalanceBefore,
            totalPendingBefore,
            "Withdraw Path 1 (A-A-A)"
        );
    }

    // Path 2: Sender: A, Controller: B, Owner: A
    // Sender (A) is an operator for Controller (B). Owner (A) has pending request under Controller (B).
    function test_withdraw_Path2_SenderIsControllerOperator() public {
        address sender = makeAddr("a-w2");
        address controller = makeAddr("b-w2");
        address owner_addr = sender; // Owner is the sender

        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        // Setup 1: Owner (A) deposits to get shares
        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        // Setup 2: Controller (B) approves Sender (A) as its ERC7540 Operator
        vm.startPrank(controller);
        vault.setOperator(sender, true);
        vm.stopPrank();

        // Request redeem (Caller: A (operator for B), Controller: B, Owner: A)
        vm.prank(sender);
        uint256 requestId_A_B_A = vault.requestRedeem(sharesToRequest, controller, owner_addr);

        skip(8 days); // Pass withdrawal delay

        uint256 assetsToWithdraw = vault.maxWithdraw(controller); // maxWithdraw for controller B
        uint256 receiverAssetBalanceBefore = underlying.balanceOf(owner_addr); // A is also the receiver
        uint256 vaultSharesBalanceBefore = vault.balanceOf(address(vault));
        uint256 totalPendingBefore = vault.getTotalPendingRedemption();

        vm.startPrank(sender); // Sender is A (who is operator for controller B)
        vm.startSnapshotGas("SLAYVaultV2", "withdraw()_sender_a_controller_b_owner_a");
        vault.withdraw(assetsToWithdraw, owner_addr, controller); // Receiver is A, Controller is B
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertWithdrawSuccess(
            sender,
            owner_addr,
            controller,
            assetsToWithdraw,
            sharesToRequest,
            receiverAssetBalanceBefore,
            vaultSharesBalanceBefore,
            totalPendingBefore,
            "Withdraw Path 2 (A-B-A, A is op for B)"
        );
    }

    // Path 3: Sender: A, Controller: A, Owner: B (Owner Approved Sender for shares)
    // Sender (A) is the Controller (A). Owner (B) has shares and approved A.
    // ERC20 approve from B to A is NOT relevant for withdraw permission, only for requestRedeem.
    function test_withdraw_Path3_SenderIsController_OwnerDifferent_OwnerApprovedSender() public {
        address sender = makeAddr("a-w3");
        address controller = sender; // Controller is the sender
        address owner_addr = makeAddr("b-w3");

        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        // Setup 1: Owner (B) deposits to get shares
        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        // Setup 2: Owner (B) approves Sender (A) on the vault shares
        vm.startPrank(owner_addr);
        vault.approve(sender, sharesToRequest); // This approval is for the shares of B
        vm.stopPrank();

        vm.prank(sender);
        vault.requestRedeem(sharesToRequest, controller, owner_addr);

        skip(8 days); // Pass withdrawal delay

        uint256 assetsToWithdraw = vault.maxWithdraw(controller); // Max withdraw for Controller A
        uint256 receiverAssetBalanceBefore = underlying.balanceOf(sender); // A is also the receiver in this test
        uint256 vaultSharesBalanceBefore = vault.balanceOf(address(vault));
        uint256 totalPendingBefore = vault.getTotalPendingRedemption();

        vm.startPrank(sender); // Sender is A (who is the controller)
        vm.startSnapshotGas("SLAYVaultV2", "withdraw()_sender_a_controller_a_owner_b_approved");
        vault.withdraw(assetsToWithdraw, sender, controller); // Receiver is A, Controller is A
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertWithdrawSuccess(
            sender,
            sender,
            controller,
            assetsToWithdraw,
            sharesToRequest,
            receiverAssetBalanceBefore,
            vaultSharesBalanceBefore,
            totalPendingBefore,
            "Withdraw Path 3 (A-A-B, B approved A for shares)"
        );
    }

    // Path 4: Sender: A, Controller: A, Owner: B (Owner Permitted Sender for shares)
    // Sender (A) is the Controller (A). Owner (B) has shares and permitted A.
    // ERC20 permit from B to A is NOT relevant for withdraw permission, only for requestRedeem.
    function test_withdraw_Path4_SenderIsController_OwnerDifferent_OwnerPermittedSender() public {
        address sender = makeAddr("a-w4");
        address controller = sender;
        uint256 owner_private_key = 0xee1159a48cad1ae7ce19ef7faadb21b926b9dadf5f4d16b5385117ca17c30b63;
        address owner_addr = vm.addr(owner_private_key);

        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 100 * 10 ** vault.decimals();

        // Setup 1: Owner (B) deposits to get shares
        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        // Setup 2: Owner (B) signs a permit for Sender (A) on vault shares
        uint256 nonce = vault.nonces(owner_addr);
        uint256 deadline = block.timestamp + 3600;

        bytes32 DOMAIN_SEPARATOR = vault.DOMAIN_SEPARATOR();
        bytes32 structHash = keccak256(
            abi.encode(
                keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"),
                owner_addr,
                sender,
                sharesToRequest,
                nonce,
                deadline
            )
        );
        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", DOMAIN_SEPARATOR, structHash));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(owner_private_key, digest);

        // Request redeem (Caller: A, Controller: A, Owner: B) using the permit signature
        vm.prank(sender);
        vault.permit(owner_addr, sender, sharesToRequest, deadline, v, r, s); // A submits B's permit signature
        vault.requestRedeem(sharesToRequest, controller, owner_addr);

        skip(8 days); // Pass withdrawal delay

        uint256 assetsToWithdraw = vault.maxWithdraw(controller);
        uint256 receiverAssetBalanceBefore = underlying.balanceOf(sender);
        uint256 vaultSharesBalanceBefore = vault.balanceOf(address(vault));
        uint256 totalPendingBefore = vault.getTotalPendingRedemption();

        vm.startPrank(sender); // Sender is A (who is the controller)
        vm.startSnapshotGas("SLAYVaultV2", "withdraw()_sender_a_controller_a_owner_b_permitted");
        vault.withdraw(assetsToWithdraw, sender, controller);
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertWithdrawSuccess(
            sender,
            sender,
            controller,
            assetsToWithdraw,
            sharesToRequest,
            receiverAssetBalanceBefore,
            vaultSharesBalanceBefore,
            totalPendingBefore,
            "Withdraw Path 4 (A-A-B, B permitted A for shares)"
        );
    }

    // Path 5: Sender: A, Controller: A, Owner: B (Sender is Owner's ERC7540 Operator)
    // Sender (A) is the Controller (A). Owner (B) has shares and set A as its operator.
    // The owner's operator status for the sender is relevant for `requestRedeem` here,
    // but for `withdraw`, since the `sender` IS the `controller`, the `onlyControllerOrOperator` check passes directly.
    function test_withdraw_Path5_SenderIsController_OwnerDifferent_SenderIsOwnerOperator() public {
        address sender = makeAddr("a-w5");
        address controller = sender; // Controller is the sender
        address owner_addr = makeAddr("b-w5");

        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        // Setup 1: Owner (B) deposits to get shares
        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        vm.prank(owner_addr);
        vault.setOperator(sender, true);
        vm.prank(sender);
        vault.requestRedeem(sharesToRequest, controller, owner_addr);

        skip(8 days); // Pass withdrawal delay

        uint256 assetsToWithdraw = vault.maxWithdraw(controller);
        uint256 receiverAssetBalanceBefore = underlying.balanceOf(sender);
        uint256 vaultSharesBalanceBefore = vault.balanceOf(address(vault));
        uint256 totalPendingBefore = vault.getTotalPendingRedemption();

        vm.startPrank(sender); // Sender is A (who is the controller)
        vm.startSnapshotGas("SLAYVaultV2", "withdraw()_sender_a_controller_a_owner_b");
        vault.withdraw(assetsToWithdraw, sender, controller);
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertWithdrawSuccess(
            sender,
            sender,
            controller,
            assetsToWithdraw,
            sharesToRequest,
            receiverAssetBalanceBefore,
            vaultSharesBalanceBefore,
            totalPendingBefore,
            "Withdraw Path 5 (A-A-B, A is op for B (irrelevant for withdraw permission))"
        );
    }

    // Path 6: Sender: A, Controller: C, Owner: B
    // Sender (A) is NOT the Controller (C). Sender (A) must be an ERC7540 Operator for Controller (C).
    // Owner (B) is different from both. This is a common scenario for a controller withdrawing for a staker.
    function test_withdraw_Path6_SenderIsControllerOperator_OwnerDifferent() public {
        address sender = makeAddr("a-w6"); // This is 'a'
        address controller = makeAddr("c-w6"); // This is 'c'
        address owner_addr = makeAddr("b-w6"); // This is 'b'

        uint256 initialDeposit = 100 * 10 ** vault.decimals();
        uint256 sharesToRequest = 50 * 10 ** vault.decimals();

        // Setup 1: Owner (B) deposits to get shares
        btcToken.mint(owner_addr, initialDeposit);
        vm.startPrank(owner_addr);
        btcToken.approve(address(vault), type(uint256).max);
        vault.deposit(initialDeposit, owner_addr);
        vm.stopPrank();

        // Setup 2: Controller (C) approves Sender (A) as its ERC7540 Operator
        vm.startPrank(controller);
        vault.setOperator(sender, true); // C approves A as operator
        vm.stopPrank();

        // Setup 3: Owner (B) must allow Controller (C) to request redeem on its behalf, or C is B's operator.
        // For simplicity here, let's assume Controller C is an operator for Owner B
        // OR Owner B approves Controller C to transfer its shares for the requestRedeem.
        // Let's go with B approving C to request redeem (most flexible for testing).
        vm.prank(owner_addr);
        vault.setOperator(controller, true); // B sets C as its operator

        // Request redeem (Caller: C (operator for B), Controller: C, Owner: B)
        // Or, if B approves C for shares, then C can call requestRedeem for B.
        vm.prank(controller); // C is calling requestRedeem, and C is operator for B
        vault.requestRedeem(sharesToRequest, controller, owner_addr);

        skip(8 days); // Pass withdrawal delay

        uint256 assetsToWithdraw = vault.maxWithdraw(controller); // Max withdraw for Controller C
        uint256 receiverAssetBalanceBefore = underlying.balanceOf(owner_addr); // Receiver is B
        uint256 vaultSharesBalanceBefore = vault.balanceOf(address(vault));
        uint256 totalPendingBefore = vault.getTotalPendingRedemption();

        vm.startPrank(sender); // Sender is A (who is operator for controller C)
        vm.startSnapshotGas("SLAYVaultV2", "withdraw()_sender_a_controller_c_owner_b");
        vault.withdraw(assetsToWithdraw, owner_addr, controller); // Receiver is B, Controller is C
        vm.stopSnapshotGas();
        vm.stopPrank();

        _assertWithdrawSuccess(
            sender,
            owner_addr,
            controller,
            assetsToWithdraw,
            sharesToRequest,
            receiverAssetBalanceBefore,
            vaultSharesBalanceBefore,
            totalPendingBefore,
            "Withdraw Path 6 (A-C-B, A is op for C)"
        );
    }
}
