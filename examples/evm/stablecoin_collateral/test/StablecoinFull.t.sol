// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC4626} from "@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

import {TestSuiteV2} from "./TestSuiteV2.sol";

import "../src/PositionLocker.sol"; // PL
import "../src/ConversionGateway.sol"; // CG
import "../src/Connector.sol"; // ExternalVaultConnector
import "./MockERC20.sol";

contract Simple4626 is ERC20, ERC4626 {
    uint8 private immutable _dec;

    constructor(ERC20 underlying, string memory name_, string memory symbol_)
        ERC20(name_, symbol_)
        ERC4626(underlying)
    {
        _dec = underlying.decimals();
    }

    function decimals() public view override(ERC20, ERC4626) returns (uint8) {
        return _dec;
    }
}

contract MockWrapper1to1 is IWrapper1to1 {
    MockERC20 public immutable _base;
    MockERC20 public immutable _wrapped;
    uint256 public unwrapNextOut; // for negative tests; unused here

    constructor(MockERC20 base_, MockERC20 wrapped_) {
        _base = base_;
        _wrapped = wrapped_;
    }

    function base() external view override returns (address) {
        return address(_base);
    }

    function wrapped() external view override returns (address) {
        return address(_wrapped);
    }

    // Pull base and mint wrapped 1:1 to caller
    function wrap(uint256 amount) external override returns (uint256 out) {
        require(amount > 0, "wrap/zero");
        _base.transferFrom(msg.sender, address(this), amount);
        _wrapped.mint(msg.sender, amount);
        return amount;
    }
    // Pull wrapped and mint base 1:1 to caller

    function unwrap(uint256 amount) external override returns (uint256 out) {
        require(amount > 0, "unwrap/zero");
        _wrapped.transferFrom(msg.sender, address(this), amount);
        _base.mint(msg.sender, amount);
        return amount;
    }
}

contract StablecoinFullIntegrationTest is Test, TestSuiteV2 {
    // actors
    address public gov = makeAddr("gov"); // governance
    address public operator = makeAddr("operator"); // keeper (also operator in this test)
    address public alice = makeAddr("alice");

    // base & wrapped
    MockERC20 public BASE; // e.g., WBTC (8 decimals)
    MockERC20 public WRAPPED; // 1:1 wrapper token (8 decimals)

    // SatLayer vault + PL + CG
    SLAYVaultV2 public vault;
    PositionLocker public pl;
    ConversionGateway public cg;

    MockWrapper1to1 public wrapper;
    Simple4626 public extVaultWrapped; // external 4626 vault whose asset = WRAPPED
    Simple4626 public extVaultBase; // external 4626 vault whose asset = BASE
    ExternalVaultConnector public connWrapped; // connector targeting extVaultWrapped
    ExternalVaultConnector public connBase; // connector targeting extVaultBase

    // strategy ids
    bytes32 constant STRAT_WRAP = keccak256("ROUTE_WRAP");
    bytes32 constant STRAT_IDENT = keccak256("ROUTE_IDENTITY");
    StrategyId constant STRAT_WRAP_ID = StrategyId.wrap(STRAT_WRAP);
    StrategyId constant STRAT_IDENT_ID = StrategyId.wrap(STRAT_IDENT);

    function setUp() public override {
        /* --- tokens --- */
        BASE = new MockERC20("Wrapped BTC", "WBTC", 8);
        WRAPPED = new MockERC20("wWBTC", "wWBTC", 8);
        TestSuiteV2.setUp();
        // Register an operator
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        // Create vault
        vm.prank(operator);
        vault = vaultFactory.create(BASE);

        // sanity
        assertEq(vault.delegated(), operator);

        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        /* --- PL (governance/operator is the vault.delegated() = operator) --- */
        pl = new PositionLocker(vault);
        // grant CG role later when wiring

        /* --- CG with base and PL wired --- */
        cg = new ConversionGateway(gov, operator, operator, address(pl), IERC20(address(BASE)));

        /* --- External infra --- */
        wrapper = new MockWrapper1to1(BASE, WRAPPED);
        extVaultWrapped = new Simple4626(ERC20(address(WRAPPED)), "ext vWBTC", "ext-vWBTC");
        extVaultBase = new Simple4626(ERC20(address(BASE)), "ext vBASE", "ext-vBASE");

        // connectors
        connWrapped = new ExternalVaultConnector(gov, address(cg), IERC4626(address(extVaultWrapped)));
        connBase = new ExternalVaultConnector(gov, address(cg), IERC4626(address(extVaultBase)));

        /* --- Strategy config in CG --- */
        // wrap 1:1 then deposit to ext 4626 (wrapped)
        vm.prank(gov);
        cg.setStrategyWrap(STRAT_WRAP, address(wrapper), address(connWrapped));

        // identity (no wrapper) deposit base directly to ext 4626 (base)
        vm.prank(gov);
        cg.setStrategyWrap(STRAT_IDENT, address(0), address(connBase));

        //Modifying caps to allow tests
        vm.prank(operator);
        pl.setCaps(5_000, 5_000, 5_000, 1 days);

        /* --- Wire CG into PL and unpause PL --- */
        vm.prank(operator);
        pl.setConversionGateway(address(cg)); // grants ROLE_CG inside PL
        vm.prank(operator);
        pl.setStrategyEnabled(STRAT_WRAP_ID, true);
        vm.prank(operator);
        pl.setStrategyEnabled(STRAT_IDENT_ID, true);
        vm.prank(operator);
        pl.setPaused(false);

        // labels for easier debugging
        vm.label(address(vault), "SLAYVault");
        vm.label(address(pl), "PL");
        vm.label(address(cg), "CG");
        vm.label(address(wrapper), "Wrapper");
        vm.label(address(connWrapped), "ConnWrapped");
        vm.label(address(connBase), "ConnBase");
        vm.label(address(extVaultWrapped), "ExtVaultWrapped");
        vm.label(address(extVaultBase), "ExtVaultBase");
        vm.label(address(BASE), "BASE");
        vm.label(address(WRAPPED), "WRAPPED");
    }

    /* =========================================================
     *  Helper: user deposits BASE into SatLayer vault, then opt-in
     * ========================================================= */
    function _userDepositAndOptIn(address user, uint256 baseAssets, StrategyId strat)
        internal
        returns (uint256 shares)
    {
        // fund user
        BASE.mint(user, baseAssets);

        vm.startPrank(user);
        // deposit to SatLayer vault
        BASE.approve(address(vault), type(uint256).max);
        shares = vault.deposit(baseAssets, user);

        // approve PL to pull shares and opt-in
        ERC20(address(vault)).approve(address(pl), type(uint256).max);
        pl.optIn(shares, strat);
        vm.stopPrank();
    }

    /* =========================================================
     *  Test: wrap full path
     *   alice: deposit → opt-in → operator request → claim (PL->CG) →
     *   CG wrap+deposit → unwind all → CG unwrap → PL repay+restake → opt-out
     * ========================================================= */
    function test_Wrap_FullFlow() public {
        uint256 depositAmt = 500e8; // 500 WBTC (8 decimals)
        uint256 shares = _userDepositAndOptIn(alice, depositAmt, STRAT_WRAP_ID);

        // operator draws half the shares
        uint256 reqShares = shares / 2;

        vm.startPrank(operator);
        uint256 reqId = pl.requestFor(alice, reqShares, STRAT_WRAP_ID);
        vm.stopPrank();

        // progress time to make claimable
        skip(7 days);

        // Claim: vault pays underlaying to CG; PL books debt; CG wraps & deposits to connector (per-user)
        vm.prank(operator);
        uint256 assetsOut = pl.claimTo(reqId);
        assertGt(assetsOut, 0, "claimed base > 0");

        // connector shows user entitlement in wrapped units (1:1)
        assertEq(connWrapped.assetsOf(alice), assetsOut, "entitlement == claimed");

        // Now operator unwinds ALL entitlement back to base and restakes to PL
        vm.prank(operator);
        cg.unwindWrapAny(alice, STRAT_WRAP, type(uint256).max);

        // After repayAndRestake, user debt should be ~0 and shares increased
        // unlockable should be full allocation for that strategy
        uint256 unlockable = pl.unlockable(alice, STRAT_WRAP_ID);
        (uint256 totalShares2,,,) = pl.userTotals(alice);
        assertEq(unlockable, totalShares2, "all shares unlockable after full unwind");

        // User can now opt out entirely from the strategy
        StrategyId[] memory arr = new StrategyId[](1);
        arr[0] = STRAT_WRAP_ID;

        vm.prank(alice);
        pl.optOutAll(arr);
        assertEq(ERC20(address(vault)).balanceOf(alice), totalShares2, "user received shares back");
    }

    /* =========================================================
     *  Test: identity full path (no wrapper)
     * ========================================================= */
    function test_Identity_FullFlow() public {
        uint256 depositAmt = 320e8;
        uint256 shares = _userDepositAndOptIn(alice, depositAmt, STRAT_IDENT_ID);

        // request ~40% of shares
        uint256 reqShares = (shares * 2) / 5;

        vm.startPrank(operator);
        uint256 reqId = pl.requestFor(alice, reqShares, STRAT_IDENT_ID);
        vm.stopPrank();

        skip(7 days);

        vm.prank(operator);
        uint256 assetsOut = pl.claimTo(reqId);
        assertGt(assetsOut, 0);

        // entitlement held directly in BASE connector (identity)
        assertEq(connBase.assetsOf(alice), assetsOut);

        // unwind half first, then the rest
        vm.prank(operator);
        cg.unwindWrapAny(alice, STRAT_IDENT, assetsOut / 2);

        // some debt remains; not fully unlockable
        uint256 unlockable1 = pl.unlockable(alice, STRAT_IDENT_ID);
        (uint256 totalShares,,,) = pl.userTotals(alice);
        assertLt(unlockable1, totalShares);

        // unwind remaining
        vm.prank(operator);
        cg.unwindWrapAny(alice, STRAT_IDENT, type(uint256).max);

        // now fully unlockable
        uint256 unlockable2 = pl.unlockable(alice, STRAT_IDENT_ID);
        (totalShares,,,) = pl.userTotals(alice);
        assertEq(unlockable2, totalShares);

        // User can now opt out entirely from the strategy
        StrategyId[] memory arr = new StrategyId[](1);
        arr[0] = STRAT_IDENT_ID;
        vm.prank(alice);
        pl.optOutAll(arr);
    }
}
