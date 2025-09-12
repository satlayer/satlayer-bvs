// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC4626} from "@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

import {TestSuiteV2} from "@satlayer/contracts/test/TestSuiteV2.sol";
import {MockERC20} from "@satlayer/contracts/test/MockERC20.sol";

import "../src/PositionLocker.sol"; // PL
import "../src/ConversionGateway.sol"; // CG
import "../src/ExternalVaultConnector.sol"; // ExternalVaultConnector

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

contract MockOracle is IPriceOracle {
    mapping(address => uint256) public px; // token => price 1e8

    function set(address token, uint256 p1e8) external {
        px[token] = p1e8;
    }

    function price(address token) external view returns (uint256) {
        return px[token];
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
    MockERC20 public DEBT; // e.g., USDC (6d)

    // SatLayer vault + PL + CG
    SLAYVaultV2 public vault;
    PositionLocker public pl;
    ConversionGateway public cg;
    MockOracle public oracle;

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
        vault = vaultFactory.create(BASE, "test", "T");

        // sanity
        assertEq(vault.delegated(), operator);

        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        /* --- PL (governance/operator is the vault.delegated() = operator) --- */
        pl = new PositionLocker(vault);
        // grant CG role later when wiring

        oracle = new MockOracle();

        // prices (1e8): BTC = 50,000; USDC = 1
        oracle.set(address(BASE), 50_000 * 1e8);
        oracle.set(address(DEBT), 1e8);

        /* --- CG with base and PL wired --- */
        cg = new ConversionGateway(gov, operator, operator, address(pl), oracle, IERC20(address(BASE)));

        /* --- External infra --- */
        wrapper = new MockWrapper1to1(BASE, WRAPPED);
        extVaultWrapped = new Simple4626(ERC20(address(WRAPPED)), "ext vWBTC", "ext-vWBTC");
        extVaultBase = new Simple4626(ERC20(address(BASE)), "ext vBASE", "ext-vBASE");

        // connectors
        connWrapped = new ExternalVaultConnector(gov, address(cg), IERC4626(address(extVaultWrapped)));
        connBase = new ExternalVaultConnector(gov, address(cg), IERC4626(address(extVaultBase)));

        /* --- Strategy config in CG --- */

        // sensible defaults for deposit safety
        ConversionGateway.DepositSafety memory depSafeWrap = ConversionGateway.DepositSafety({
            redeemToleranceBps: 25, // allow 0.25% shortfall on 4626 redeem
            unwrapMinOutBps: 9950, // require at least 99.5% back on unwrap
            emergencyMode: false, // normal operation
            emergencyRedeemBps: 500, // allow up to 5% shortfall if emergencyMode = true
            emergencyUnwrapBps: 500 // allow up to 5% unwrap loss if emergencyMode = true
        });

        ConversionGateway.DepositSafety memory depSafeIdent = ConversionGateway.DepositSafety({
            redeemToleranceBps: 10, // tighter: 0.10% slack
            unwrapMinOutBps: 10_000, // exactly 100% (identity path has no unwrap)
            emergencyMode: false,
            emergencyRedeemBps: 200, // 2% slack in emergency
            emergencyUnwrapBps: 0 // unused since no wrapper
        });

        // --- WRAP strategy (base -> wrap 1:1 -> deposit wrapped) ---
        {
            ConversionGateway.StrategyCfg memory s;
            s.kind = ConversionGateway.RouteKind.DepositWrap1to1;
            s.deposit = ConversionGateway.DepositCfg({
                wrapper: address(wrapper),
                connector: address(connWrapped),
                safety: depSafeWrap
            });
            // leave s.borrow as empty/defaults
            vm.prank(gov);
            cg.setStrategy(STRAT_WRAP, s);
        }

        // --- IDENTITY strategy (deposit base directly) ---
        {
            ConversionGateway.StrategyCfg memory s;
            s.kind = ConversionGateway.RouteKind.DepositIdentity;
            s.deposit =
                ConversionGateway.DepositCfg({wrapper: address(0), connector: address(connBase), safety: depSafeIdent});
            vm.prank(gov);
            cg.setStrategy(STRAT_IDENT, s);
        }

        //cg.setStrategyWrap(STRAT_IDENT, address(0), address(connBase));

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
        uint256 assetsOut = pl.claimTo(reqId, "");
        assertGt(assetsOut, 0, "claimed base > 0");
        assertEq(connWrapped.assetsOf(alice), assetsOut, "entitlement == claimed");

        //Simulating yield in the external protocol vault
        uint256 yield = assetsOut / 10; // +10%

        MockERC20(WRAPPED).mint(address(extVaultWrapped), yield);

        // connector shows user entitlement in wrapped units (1:1) (deposited + yield)
        uint256 entAfter = connWrapped.assetsOf(alice);
        assertGt(entAfter, assetsOut, "yield should increase entitlement");

        // Now operator unwinds ALL entitlement back to base and restakes to PL
        vm.prank(operator);
        cg.unwindDepositAny(alice, STRAT_WRAP, type(uint256).max);

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
        uint256 assetsOut = pl.claimTo(reqId, "");
        assertGt(assetsOut, 0);

        // entitlement held directly in BASE connector (identity)
        assertEq(connBase.assetsOf(alice), assetsOut);

        //Simulating yield in the external protocol vault
        uint256 yield = assetsOut / 10; // +10%

        MockERC20(BASE).mint(address(extVaultBase), yield);

        // connector shows user entitlement  (deposited + yield)
        uint256 entAfter = connBase.assetsOf(alice);
        assertGt(entAfter, assetsOut, "yield should increase entitlement");

        // unwind half first, then the rest
        vm.prank(operator);
        cg.unwindDepositAny(alice, STRAT_IDENT, assetsOut / 2);

        // some debt remains; not fully unlockable
        uint256 unlockable1 = pl.unlockable(alice, STRAT_IDENT_ID);
        (uint256 totalShares,,,) = pl.userTotals(alice);
        assertLt(unlockable1, totalShares);

        // unwind remaining
        vm.prank(operator);
        cg.unwindDepositAny(alice, STRAT_IDENT, type(uint256).max);

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

/* ------------------------- Minimal Mock Adapter -------------------------
 * Venue adapter that:
 *  - tracks "supplied base collateral" and "venue debt"
 *  - on borrow, mints `debtAsset` to the caller (CG)
 *  - on repay, pulls `debtAsset` from caller, reduces venueDebt
 *  - on withdrawCollateral, sends base back to caller (CG)
 * Everything is 1:1 and instantaneous for testing.
 */
contract MockBorrowVenueAdapter is IBorrowVenueAdapter {
    address public immutable base; // collateral token
    address public immutable debtAsset; // debt token

    uint256 public collBal; // total collateral credited to the vault
    uint256 public debtBal; // total outstanding venue debt

    constructor(address _base, address _debt) {
        base = _base;
        debtAsset = _debt;
    }

    function supplyCollateral(address collateral, uint256 amount, bytes calldata) external {
        require(collateral == base, "coll/asset");
        // pull base from caller and credit
        MockERC20(base).transferFrom(msg.sender, address(this), amount);
        collBal += amount;
    }

    function withdrawCollateral(address collateral, uint256 amount, bytes calldata) external returns (uint256) {
        require(collateral == base, "wd/asset");
        require(amount <= collBal, "wd/excess");
        collBal -= amount;
        MockERC20(base).transfer(msg.sender, amount);
        return amount;
    }

    function collateralBalance(address collateral) external view returns (uint256) {
        require(collateral == base, "bal/asset");
        return collBal;
    }

    function borrow(address token, uint256 amount, bytes calldata) external {
        require(token == debtAsset, "borrow/asset");
        debtBal += amount;
        // mint debt to borrower (CG)
        MockERC20(debtAsset).mint(msg.sender, amount);
    }

    function repay(address token, uint256 amount, bytes calldata) external returns (uint256 repaid) {
        require(token == debtAsset, "repay/asset");
        // pull from caller up to debtBal
        uint256 want = amount > debtBal ? debtBal : amount;
        if (want > 0) {
            MockERC20(debtAsset).transferFrom(msg.sender, address(this), want);
            debtBal -= want;
        }
        return want;
    }

    function debtBalance(address token) external view returns (uint256) {
        require(token == debtAsset, "debt/asset");
        return debtBal;
    }

    function getRiskSignals(address debtAssetadd)
        external
        view
        override
        returns (bool hasApr, uint256 aprBps, bool haveHf, uint256 hfBps)
    {
        return (false, 0, false, 0); // default: no signals
    }
}

/* ----------------------- Borrow Flow Integration ----------------------- */
contract BorrowFlowIntegrationTest is Test, TestSuiteV2 {
    // actors
    address public gov = makeAddr("gov");
    address public operator = makeAddr("operator"); // keeper/operator
    address public alice = makeAddr("alice");

    // tokens
    MockERC20 public BASE; // e.g., WBTC (8 decimals)
    MockERC20 public DEBT; // e.g., USDC (6 decimals)
    MockOracle public oracle;

    // core SatLayer
    SLAYVaultV2 public vault;
    PositionLocker public pl;
    ConversionGateway public cg;

    // borrow stack
    MockBorrowVenueAdapter public adapter;
    ExternalVaultConnector public debtConn; // parks borrowed token per-user

    // strategy id
    bytes32 constant STRAT_BORROW = keccak256("ROUTE_BORROW");
    StrategyId constant STRAT_BORROW_ID = StrategyId.wrap(STRAT_BORROW);

    function setUp() public override {
        // tokens
        BASE = new MockERC20("Wrapped BTC", "WBTC", 8);
        DEBT = new MockERC20("USD Coin", "USDC", 6);

        // SatLayer testbed
        TestSuiteV2.setUp();

        // register operator
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        // create SatLayer vault (BASE)
        vm.prank(operator);
        vault = vaultFactory.create(BASE, "test", "T");
        assertEq(vault.delegated(), operator);
        vm.prank(owner);
        router.setVaultWhitelist(address(vault), true);

        // PL
        pl = new PositionLocker(vault);
        oracle = new MockOracle();
        // prices (1e8): BTC = 50,000; USDC = 1
        oracle.set(address(BASE), 50_000 * 1e8);
        oracle.set(address(DEBT), 1e8);

        // CG wired to PL
        cg = new ConversionGateway(gov, operator, operator, address(pl), oracle, IERC20(address(BASE)));

        // adapter + connector
        adapter = new MockBorrowVenueAdapter(address(BASE), address(DEBT));
        // connector is an ERC-4626 adapter; here we "park" DEBT per-user (no external yield, but interface-compatible)
        debtConn = new ExternalVaultConnector(
            gov, address(cg), IERC4626(address(new Simple4626(ERC20(address(DEBT)), "ext debt", "extDEBT")))
        );

        ConversionGateway.BorrowSafety memory bSafe = ConversionGateway.BorrowSafety({
            redeemToleranceBps: 50, // 0.50% slack when redeeming debt from connector
            withdrawSlippageBps: 50, // shave 0.50% from pro-rata collateral withdrawal
            maxAprBps: 1500, // block new borrows if APR > 15%
            minHfBps: 1200, // block new borrows if HF < 120% (safety margin)
            emergencyMode: false,
            emergencyRedeemBps: 300, // allow 3% shortfall in emergency
            emergencyWithdrawBps: 300 // allow 3% extra shave in emergency
        });

        // configure strategy (BorrowVsBase)
        ConversionGateway.StrategyCfg memory s;
        s.kind = ConversionGateway.RouteKind.BorrowVsBase;
        s.borrow = ConversionGateway.BorrowCfg({
            adapter: address(adapter),
            debtAsset: address(DEBT),
            borrowedConnector: address(debtConn),
            maxBorrowBps: 7000, // 70% LTV cap
            safety: bSafe
        });
        // add other fields here if your BorrowCfg has more

        vm.prank(gov);
        cg.setStrategy(STRAT_BORROW, s);

        // caps to allow flow
        vm.prank(operator);
        pl.setCaps(5_000, 5_000, 5_000, 1 days);

        // Wire CG into PL & enable strategy
        vm.prank(operator);
        pl.setConversionGateway(address(cg));
        vm.prank(operator);
        pl.setStrategyEnabled(STRAT_BORROW_ID, true);
        vm.prank(operator);
        pl.setPaused(false);

        // labels
        vm.label(address(vault), "SLAYVault");
        vm.label(address(pl), "PL");
        vm.label(address(cg), "CG");
        vm.label(address(adapter), "Adapter");
        vm.label(address(debtConn), "DebtConnector");
        vm.label(address(BASE), "BASE");
        vm.label(address(DEBT), "DEBT");
    }

    /* ------------ helper: user deposit + opt-in to borrow strategy ------------ */
    function _userDepositAndOptIn(address user, uint256 baseAssets, StrategyId strat)
        internal
        returns (uint256 shares)
    {
        BASE.mint(user, baseAssets);

        vm.startPrank(user);
        BASE.approve(address(vault), type(uint256).max);
        shares = vault.deposit(baseAssets, user);

        ERC20(address(vault)).approve(address(pl), type(uint256).max);
        pl.optIn(shares, strat);
        vm.stopPrank();
    }

    /* ============================== The Test ============================== */
    function test_Borrow_FullFlow() public {
        // 1) user deposits & opts-in
        uint256 depositAmt = 1_000e8; // 1,000 WBTC (test-large to get easy math)
        uint256 shares = _userDepositAndOptIn(alice, depositAmt, STRAT_BORROW_ID);

        // 2) operator requests & claims half the shares
        uint256 reqShares = shares / 2;
        vm.startPrank(operator);
        uint256 reqId = pl.requestFor(alice, reqShares, STRAT_BORROW_ID);
        vm.stopPrank();

        skip(7 days); // make claimable

        // 3) claim: vault pays BASE to CG; PL books debt; CG opens/extends the borrow position
        //    We pass params that override borrow bps = 50% (matches strategy cap)
        bytes memory params = abi.encode(uint16(5000), bytes(""), bytes(""), uint256(0));
        vm.prank(operator);
        uint256 claimedBase = pl.claimTo(reqId, params); // <- use the overload with params; if you only have 3-arg, call that one and let CG default.

        assertGt(claimedBase, 0, "claimed base > 0");

        // Adapter should now hold claimedBase as collateral
        assertEq(adapter.collateralBalance(address(BASE)), claimedBase, "collateral at venue");

        // Debt tokens should have been borrowed and parked per-user in the DEBT connector
        // Because our adapter ignores oracle and the CG uses override=5000bps, the adapter mints exactly 50% of baseIn *in DEBT units*.
        // Since DEBT has different decimals, we only check >0. For reproducible math, you can normalize or set both to same decimals.
        uint256 entitlement = debtConn.assetsOf(alice);
        assertGt(entitlement, 0, "user debt entitlement > 0");

        //Simulating yield in the external protocol vault
        uint256 yield = entitlement / 10; // +10%

        MockERC20(DEBT).mint(address(debtConn.targetVault()), yield);

        // connector shows user entitlement
        uint256 entAfter = debtConn.assetsOf(alice);
        assertGt(entAfter, entitlement, "yield should increase entitlement");

        // 4) unwind: redeem ALL user's DEBT from the connector, repay venue, withdraw proportional BASE, restake to PL
        // Tolerate a tiny rounding/slippage (e.g. 1 bp)
        uint256 tolBps = 1; // 0.01%
        uint256 connectorMinOut = (entAfter * (10_000 - tolBps)) / 10_000;
        uint256 minCollateralOut = 1; // accept any positive base after rounding shaves

        vm.startPrank(operator);
        (uint256 baseOut, uint256 repaidDebt, uint256 redeemedDebt) =
            cg.unwindBorrow(alice, STRAT_BORROW, type(uint256).max, minCollateralOut, bytes(""), connectorMinOut);

        uint256 leftover = debtConn.assetsOf(alice);

        if (leftover > 0) {
            // allow any amount out, just clear dust
            cg.unwindBorrow(alice, STRAT_BORROW, type(uint256).max, 1, bytes(""), 0);
        }
        uint256 entitlementAfter = debtConn.assetsOf(alice);

        pl.setDustAndBuffer(100_000, 0);

        assertEq(entitlementAfter, 0, "entitlement cleared");

        assertGt(repaidDebt, 0, "repaid > 0");
        assertEq(adapter.debtBalance(address(DEBT)), 0, "venue debt cleared");
        assertGt(baseOut, 0, "withdrew base");

        // 5) after PL.repayAndRestake inside CG, user debt in PL should go down and shares grow;
        //    Finally, user should be able to fully unlock/opt-out again for that strategy.
        uint256 unlockable = pl.unlockable(alice, STRAT_BORROW_ID);
        (uint256 totalShares2,,,) = pl.userTotals(alice);
        assertEq(unlockable, totalShares2, "all shares unlockable after full unwind");

        // opt out all from that strategy

        StrategyId[] memory arr = new StrategyId[](1);
        arr[0] = STRAT_BORROW_ID;
        vm.stopPrank();

        vm.prank(alice);
        pl.optOutAll(arr);
    }
}
