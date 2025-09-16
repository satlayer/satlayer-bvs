// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import "../src/PositionLocker.sol";
import "../src/ConversionGateway.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "@satlayer/contracts/test/TestSuiteV2.sol";
import {MockERC20} from "@satlayer/contracts/test/MockERC20.sol";
import {IERC20Errors} from "@openzeppelin/contracts/interfaces/draft-IERC6093.sol";
import {IERC165} from "@openzeppelin/contracts/utils/introspection/IERC165.sol";

/// @notice Minimal CG that can receive vault underlaying and later call repay
contract MockConversionGateway {
    IERC20 public immutable asset;

    event OnClaimWithStrategy(address indexed user, uint256 assets, bytes32 indexed strat, bytes params);

    constructor(IERC20 _asset) {
        asset = _asset;
    }

    // IConversionGatewayMulti
    function onClaimWithStrategy(address user, uint256 assets, bytes32 strategy, bytes calldata params) external {
        // The vault has already sent us `assets`. We just record.
        emit OnClaimWithStrategy(user, assets, strategy, params);
    }

    // helper: approve PL to pull assets back during repayAndRestake test
    function approvePL(address pl, uint256 amount) external {
        asset.approve(pl, amount);
    }
}

contract PositionLockerTest is Test, TestSuiteV2 {
    MockERC20 public underlying = new MockERC20("Wrapped Bitcoin", "WBTC", 8);
    address public immutable operator = makeAddr("Operator Y");

    SLAYVaultV2 public vault;
    PositionLocker public pl; // “PL” = PositionLocker
    MockConversionGateway public cg;

    // Strategy ids
    bytes32 constant STRAT_A_BYTES = keccak256("AAVE_USDC");
    StrategyId constant STRAT_A = StrategyId.wrap(STRAT_A_BYTES);

    address public alice = makeAddr("alice");
    address public bob = makeAddr("bob");

    function setUp() public override {
        TestSuiteV2.setUp();

        // Register an operator
        vm.startPrank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");
        vm.stopPrank();

        // Create vault
        vm.prank(operator);
        vault = vaultFactory.create(underlying, "test", "T");
        //vault = vaultFactory.create(underlying);
        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        // Deploy PL (governance = operator)
        pl = new PositionLocker(vault);

        // Conversion gateway
        cg = new MockConversionGateway(IERC20(vault.asset()));

        // Wire CG into PL and grant ROLE_CG
        vm.prank(operator);
        pl.setConversionGateway(address(cg));
        //Modifying caps to allow seamless operation
        vm.prank(operator);
        pl.setCaps(5_000, 5_000, 5_000, 1 days);

        // Enable strategy
        vm.prank(operator);
        pl.setStrategyEnabled(STRAT_A, true);

        // Unpause PL so user functions can run
        vm.prank(operator);
        pl.setPaused(false);

        // Fund users with underlying
        underlying.mint(alice, 1_000e8);
        underlying.mint(bob, 1_000e8);
    }

    function _aliceDeposits(uint256 assets) internal returns (uint256 shares) {
        vm.startPrank(alice);
        underlying.approve(address(vault), type(uint256).max);
        shares = vault.convertToShares(assets);
        vault.deposit(assets, alice);
        vm.stopPrank();
    }

    function _optInAlice(uint256 shareAmount, StrategyId strat) internal {
        // Alice must approve PL to pull vault shares
        vm.startPrank(alice);
        IERC20(address(vault)).approve(address(pl), type(uint256).max);
        pl.optIn(shareAmount, strat);
        vm.stopPrank();
    }

    function test_init_roles_and_pausing() public {
        // Pausable default per patch: paused
        vm.prank(operator);
        pl.setPaused(true);
        assertTrue(pl.paused());

        // Unpause and check again
        vm.prank(operator);
        pl.setPaused(false);
        assertFalse(pl.paused());

        // Roles synced from vault.delegated() == operator
        bytes32 RG = pl.ROLE_GOVERNANCE();
        bytes32 RK = pl.ROLE_KEEPER();
        bytes32 RP = pl.ROLE_PAUSER();
        bytes32 RCG = pl.ROLE_CG();

        assertTrue(pl.hasRole(RG, operator));
        assertTrue(pl.hasRole(RK, operator));
        assertTrue(pl.hasRole(RP, operator));
        assertTrue(pl.hasRole(RCG, address(cg)));
    }

    function test_optIn_and_unlockable_noDebt() public {
        uint256 assets = 100e8;
        _aliceDeposits(assets);

        uint256 shares = vault.balanceOf(alice);
        _optInAlice(shares, STRAT_A);

        // All shares unlockable if no debt
        uint256 unlockableShares = pl.unlockable(alice, STRAT_A);
        assertEq(unlockableShares, shares, "unlockable==allocated when no debt");

        // userTotals reflects totals
        (uint256 totalShares,,,) = pl.userTotals(alice);
        assertEq(totalShares, shares);
    }

    function test_keeper_request_claim_and_repay_full_flow() public {
        uint256 assets = 500e8;
        _aliceDeposits(assets);

        uint256 shares = vault.balanceOf(alice);
        _optInAlice(shares, STRAT_A);

        // Keeper (operator) requests a portion
        uint256 reqShares = shares / 2;
        vm.startPrank(operator);

        uint256 reqId = pl.requestFor(alice, reqShares, STRAT_A);
        vm.stopPrank();

        // requests created; now fast-forward 7 days so claim is ready
        skip(7 days);

        // Claim to CG → books user debt/transformed + sends underlaying to CG
        vm.prank(operator);
        uint256 assetsOut = pl.claimTo(reqId, "");

        // Check transformed  moved up
        (, uint256 transformedTotal,,) = pl.userTotals(alice);
        assertEq(transformedTotal, assetsOut, "user transformed updated");

        // PL global transformed moved too
        assertEq(pl.globalTransformed(), assetsOut);

        // Now CG approves PL to pull back and restake
        vm.startPrank(address(cg));
        cg.approvePL(address(pl), assetsOut);
        // repay → reduces debt, deposits back to vault for PL, increases PL-held shares for Alice
        uint256 beforePtotal;
        (beforePtotal,,,) = pl.userTotals(alice);
        uint256 outShares = pl.repayAndRestake(alice, assetsOut, STRAT_A);
        vm.stopPrank();

        // Debt should be back to 0, unlockable returns all allocated
        uint256 unlockableShares = pl.unlockable(alice, STRAT_A);
        (uint256 afterTotalShares,,,) = pl.userTotals(alice);

        assertGt(outShares, 0);
        assertEq(pl.globalTransformed(), 0, "global transformed cleared");
        assertEq(unlockableShares, afterTotalShares, "all shares unlockable again");
        assertGt(afterTotalShares, beforePtotal, "shares increased (re-deposit sharesOut)");
    }

    function test_optOutAll_blocks_when_debt_exists() public {
        uint256 assets = 200e8;
        _aliceDeposits(assets);
        uint256 shares = vault.balanceOf(alice);
        _optInAlice(shares, STRAT_A);

        // Open request to create debt on claim
        vm.prank(operator);
        uint256 reqId = pl.requestFor(alice, shares / 2, STRAT_A);
        skip(7 days);

        vm.prank(operator);
        pl.claimTo(reqId, ""); // books debt

        // Try to optOutAll while debt > dust => revert
        //StrategyId [] calldata arr = [STRAT_A];
        StrategyId[] memory arr = new StrategyId[](1);
        arr[0] = STRAT_A;

        vm.prank(alice);
        vm.expectRevert(bytes("OUTSTANDING_DEBT"));
        pl.optOutAll(arr);
    }

    function test_unlockable_withDebt_and_buffer() public {
        uint256 assets = 1_000e8;
        _aliceDeposits(assets);
        uint256 shares = vault.balanceOf(alice);
        _optInAlice(shares, STRAT_A);

        // Request an amount that will become debt later
        uint256 reqShares = vault.convertToShares(400e8);

        vm.startPrank(operator);
        uint256 reqId = pl.requestFor(alice, reqShares, STRAT_A);
        vm.stopPrank();

        skip(7 days);

        vm.prank(operator);
        uint256 assetsOut = pl.claimTo(reqId, ""); // debtAssets += assetsOut

        // Actual under-test value
        uint256 unlockableShares = pl.unlockable(alice, STRAT_A);

        // Expected math (use post-request allocation)
        uint256 allocAfterRequest = shares - reqShares;
        uint256 enc = vault.convertToShares(assetsOut);
        uint256 buffer = (enc * pl.bufferBps()) / 10_000;
        uint256 expected = allocAfterRequest > (enc + buffer) ? allocAfterRequest - (enc + buffer) : 0;

        assertEq(unlockableShares, expected, "unlockable keeps buffer over debt from remaining allocation");
    }

    function test_optOutFromStrategy_respects_unlockable() public {
        uint256 assets = 300e8;
        _aliceDeposits(assets);
        uint256 shares = vault.balanceOf(alice);
        _optInAlice(shares, STRAT_A);

        // Take on some debt so not all shares are unlockable
        vm.prank(operator);
        uint256 reqId = pl.requestFor(alice, shares / 3, STRAT_A);
        skip(7 days);
        vm.prank(operator);
        pl.claimTo(reqId, "");

        uint256 maxOut = pl.unlockable(alice, STRAT_A);
        assertGt(maxOut, 0);
        assertLt(maxOut, shares);

        // Too much → revert
        vm.startPrank(alice);
        vm.expectRevert(bytes("LOCKED_BY_DEBT"));
        pl.optOutFromStrategy(maxOut + 1, STRAT_A);

        // Exact unlockable → success
        pl.optOutFromStrategy(maxOut, STRAT_A);
        vm.stopPrank();
    }
}
