// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";

import {MockERC20} from "@satlayer/contracts/test/MockERC20.sol";
import "../src/ConversionGateway.sol";

/* ------------------------ Mocks ------------------------ */



contract MockOracle is IPriceOracle {
    mapping(address => uint256) public px; // token => price 1e8
    function set(address token, uint256 p1e8) external { px[token] = p1e8; }
    function price(address token) external view returns (uint256) { return px[token]; }
}

contract MockBorrowAdapter is IBorrowVenueAdapter {
    // simple venue accounting
    mapping(address => uint256) public collateralBal; // token => amount
    mapping(address => uint256) public debtBal;       // token => amount

    // optional knobs to simulate behavior
    uint256 public withdrawNextOut; // force a specific withdraw out for testing slippage
    uint256 public repayNextOut;    // force a specific repay out (<= input)

    function setwithdrawNextOut(uint256 v) external { withdrawNextOut = v; }

    // Collateral
    function supplyCollateral(address collateral, uint256 amount, bytes calldata) external override {
        IERC20(collateral).transferFrom(msg.sender, address(this), amount);
        collateralBal[collateral] += amount;
    }

    function withdrawCollateral(address collateral, uint256 amount, bytes calldata) external override returns (uint256) {
        uint256 out = withdrawNextOut > 0 ? withdrawNextOut : amount;
        require(collateralBal[collateral] >= out, "venue/coll<out");
        collateralBal[collateral] -= out;
        IERC20(collateral).transfer(msg.sender, out);
        withdrawNextOut = 0;
        return out;
    }

    function collateralBalance(address collateral) external view override returns (uint256) {
        return collateralBal[collateral];
    }

    // Debt
    function borrow(address debtAsset, uint256 amount, bytes calldata) external override {
        debtBal[debtAsset] += amount;
        MockERC20(debtAsset).mint(msg.sender, amount);
    }

    function repay(address debtAsset, uint256 amount, bytes calldata) external override returns (uint256 repaid) {
        uint256 want = repayNextOut > 0 ? repayNextOut : amount;
        if (want > amount) want = amount;
        uint256 d = debtBal[debtAsset];
        if (want > d) want = d;

        IERC20(debtAsset).transferFrom(msg.sender, address(this), want);
        debtBal[debtAsset] = d - want;
        repayNextOut = 0;
        return want;
    }

    function debtBalance(address debtAsset) external view override returns (uint256) {
        return debtBal[debtAsset];
    }

    function getRiskSignals(address debtAsset)
        external
        view
        override
        returns (bool hasApr, uint aprBps, bool haveHf, uint hfBps)
    {
        return (false, 0, false, 0); // default: no signals
    }
    

}



contract MockWrapper1to1 is IWrapper1to1 {
    MockERC20 public immutable _base;
    MockERC20 public immutable _wrapped;

    uint256 public unwrapNextOut;

    constructor(MockERC20 base_, MockERC20 wrapped_) {
        _base = base_;
        _wrapped = wrapped_;
    }

    function base() external view override returns (address) { return address(_base); }
    function wrapped() external view override returns (address) { return address(_wrapped); }

    function setUnwrapNextOut(uint256 v) external { unwrapNextOut = v; }

    // Pull base from caller and mint wrapped 1:1 to caller
    function wrap(uint256 amount) external override returns (uint256 out) {
        require(amount > 0, "wrap/zero");
        _base.transferFrom(msg.sender, address(this), amount);
        _wrapped.mint(msg.sender, amount);
        return amount;
    }

    // Pull wrapped from caller and mint base to caller; return out (=amount unless override set)
    function unwrap(uint256 amount) external override returns (uint256 out) {
        require(amount > 0, "unwrap/zero");
        _wrapped.transferFrom(msg.sender, address(this), amount);
        uint256 o = unwrapNextOut > 0 ? unwrapNextOut : amount;
        if (unwrapNextOut > 0) unwrapNextOut = 0;
        _base.mint(msg.sender, o);
        return o;
    }
}

contract MockExternalVaultConnector is IExternalVaultConnector {
    address public immutable _asset; // token this connector accepts / returns
    mapping(address => uint256) public ent; // per-user entitlement in asset units

    constructor(address asset_) { _asset = asset_; }

    function asset() external view override returns (address) { return _asset; }

    // Pull token from caller (CG) and credit user's entitlement 1:1
    function depositFor(address user, uint256 assets) external override returns (uint256 sharesOut) {
        require(assets > 0, "dep/zero");
        MockERC20(_asset).transferFrom(msg.sender, address(this), assets);
        ent[user] += assets;
        return assets;
    }

    // Reduce user's entitlement by min(requested, entitlement); send assets to caller (CG)
    function redeemFor(address user, uint256 requestedAssets, uint256 minAssetsOut)
        external
        override
        returns (uint256 assetsOut, uint256 sharesBurned)
    {
        uint256 e = ent[user];
        uint256 r = requestedAssets <= e ? requestedAssets : e;
        require(r >= minAssetsOut, "redeem/min");
        ent[user] = e - r;
        MockERC20(_asset).transfer(msg.sender, r);
        return (r, r);
    }

    function assetsOf(address user) external view override returns (uint256) {
        return ent[user];
    }
}

contract MockPL is IPL {
    address public lastUser;
    uint256 public lastAssets;
    bytes32 public lastStrat;
    uint256 public called;
    address public immutable asset; //underlaying


    constructor(address asset_) { asset = asset_; }

    function repayAndRestake(address user, uint256 assets, bytes32 strategy) external {
        MockERC20(asset).transferFrom(msg.sender, address(this), assets);
        lastUser = user;
        lastAssets = assets;
        lastStrat = strategy;
        called++;
    }

    function finalizeUnwind(address user, bytes32 strat) external { 
        lastUser = user;
        lastStrat = strat;

    }

 }

/* ------------------------ Tests ------------------------ */

contract ConversionGatewayTest is Test {
    // actors
    address public gov    = makeAddr("gov");
    address public keeper = makeAddr("keeper");
    address public pauser = makeAddr("pauser");
    MockPL  public pl; // gets ROLE_PL

    // tokens
    MockERC20 public BASE;     // e.g. WBTC
    MockERC20 public WRAPPED;  // 1:1 wrapper token
    MockERC20 public OTHER;    // random ERC20 to test mismatches
    MockOracle public oracle;
    MockERC20 public DEBT;   // e.g., USDC (6d)

    // system under test
    ConversionGateway public cg;

    // mocks
    MockWrapper1to1 public wrapper;
    MockExternalVaultConnector public connWrapped; // expects WRAPPED
    MockExternalVaultConnector public connBase;    // expects BASE

    bytes32 constant STRAT_WRAP  = keccak256("WRAP_DEPOSIT");
    bytes32 constant STRAT_IDENT = keccak256("IDENTITY_DEPOSIT");

    function setUp() public {
        BASE    = new MockERC20("Wrapped BTC", "WBTC", 8);
        WRAPPED = new MockERC20("wWBTC", "wWBTC", 8);
        OTHER   = new MockERC20("OTHER", "OTH", 18);

        pl = new MockPL(address(BASE));
        oracle = new MockOracle();
        // prices (1e8): BTC = 50,000; USDC = 1
        oracle.set(address(BASE), 50_000 * 1e8);
        oracle.set(address(DEBT), 1e8);

        cg = new ConversionGateway(gov, keeper, pauser, address(pl), oracle, IERC20(address(BASE)));

        wrapper     = new MockWrapper1to1(BASE, WRAPPED);
        connWrapped = new MockExternalVaultConnector(address(WRAPPED));
        connBase    = new MockExternalVaultConnector(address(BASE));

        // // --- set a WRAP strategy (wrap base -> wrapped -> deposit) ---
        // {
        //     ConversionGateway.StrategyCfg memory s;
        //     s.kind = ConversionGateway.RouteKind.DepositWrap1to1;
        //     s.deposit = ConversionGateway.DepositCfg({
        //         wrapper:   address(wrapper),
        //         connector: address(connWrapped)
        //     });
        //     // borrow cfg left empty
        //     vm.prank(gov);
        //     cg.setStrategy(STRAT_WRAP, s);
        // }

        // // --- set an IDENTITY strategy (no wrapper; deposit base directly) ---
        // {
        //     ConversionGateway.StrategyCfg memory s;
        //     s.kind = ConversionGateway.RouteKind.DepositIdentity;
        //     s.deposit = ConversionGateway.DepositCfg({
        //         wrapper:   address(0),
        //         connector: address(connBase)
        //     });
        //     vm.prank(gov);
        //     cg.setStrategy(STRAT_IDENT, s);
        // }

        // sensible defaults for deposit safety
        ConversionGateway.DepositSafety memory depSafeWrap = ConversionGateway.DepositSafety({
            redeemToleranceBps: 25,      // allow 0.25% shortfall on 4626 redeem
            unwrapMinOutBps:    9950,    // require at least 99.5% back on unwrap
            emergencyMode:      false,   // normal operation
            emergencyRedeemBps: 500,     // allow up to 5% shortfall if emergencyMode = true
            emergencyUnwrapBps: 500      // allow up to 5% unwrap loss if emergencyMode = true
        });

        ConversionGateway.DepositSafety memory depSafeIdent = ConversionGateway.DepositSafety({
            redeemToleranceBps: 10,      // tighter: 0.10% slack
            unwrapMinOutBps:    10_000,  // exactly 100% (identity path has no unwrap)
            emergencyMode:      false,
            emergencyRedeemBps: 200,     // 2% slack in emergency
            emergencyUnwrapBps: 0        // unused since no wrapper
        });


        // --- WRAP strategy (base -> wrap 1:1 -> deposit wrapped) ---
        {
            ConversionGateway.StrategyCfg memory s;
            s.kind = ConversionGateway.RouteKind.DepositWrap1to1;
            s.deposit = ConversionGateway.DepositCfg({
                wrapper:   address(wrapper),
                connector: address(connWrapped),
                safety:    depSafeWrap
            });
            // leave s.borrow as empty/defaults
            vm.prank(gov);
            cg.setStrategy(STRAT_WRAP, s);
        }

        // --- IDENTITY strategy (deposit base directly) ---
        {
            ConversionGateway.StrategyCfg memory s;
            s.kind = ConversionGateway.RouteKind.DepositIdentity;
            s.deposit = ConversionGateway.DepositCfg({
                wrapper:   address(0),
                connector: address(connBase),
                safety:    depSafeIdent
            });
            vm.prank(gov);
            cg.setStrategy(STRAT_IDENT, s);
        }

        // labels
        vm.label(address(cg), "CG");
        vm.label(address(wrapper), "Wrapper");
        vm.label(address(connWrapped), "ConnWrapped");
        vm.label(address(connBase), "ConnBase");
        vm.label(address(pl), "PL");
        vm.label(address(BASE), "BASE");
        vm.label(address(WRAPPED), "WRAPPED");
    }

    /* --------------------- setStrategy validations --------------------- */

    function test_setStrategy_validations() public {

        ConversionGateway.DepositSafety memory depSafeWrap2 = ConversionGateway.DepositSafety({
            redeemToleranceBps: 25,      // allow 0.25% shortfall on 4626 redeem
            unwrapMinOutBps:    9950,    // require at least 99.5% back on unwrap
            emergencyMode:      false,   // normal operation
            emergencyRedeemBps: 500,     // allow up to 5% shortfall if emergencyMode = true
            emergencyUnwrapBps: 500      // allow up to 5% unwrap loss if emergencyMode = true
        });

        ConversionGateway.DepositSafety memory depSafeIdent2 = ConversionGateway.DepositSafety({
            redeemToleranceBps: 10,      // tighter: 0.10% slack
            unwrapMinOutBps:    10_000,  // exactly 100% (identity path has no unwrap)
            emergencyMode:      false,
            emergencyRedeemBps: 200,     // 2% slack in emergency
            emergencyUnwrapBps: 0        // unused since no wrapper
        });

        // (1) DepositWrap1to1: wrapper base mismatch
        {
            ConversionGateway.StrategyCfg memory s;
            s.kind = ConversionGateway.RouteKind.DepositWrap1to1;
            MockWrapper1to1 badWrap = new MockWrapper1to1(OTHER, WRAPPED);
            s.deposit = ConversionGateway.DepositCfg({
                wrapper:   address(badWrap),
                connector: address(connWrapped),
                safety:    depSafeWrap2
            });
            vm.prank(gov);
            vm.expectRevert(bytes("WRAP_BASE_MISMATCH"));
            cg.setStrategy(keccak256("X"), s);
        }

        // (2) DepositWrap1to1: connector asset must equal wrapper.wrapped
        {
            ConversionGateway.StrategyCfg memory s;
            s.kind = ConversionGateway.RouteKind.DepositWrap1to1;
            MockExternalVaultConnector badConn = new MockExternalVaultConnector(address(BASE));
            s.deposit = ConversionGateway.DepositCfg({
                wrapper:   address(wrapper),
                connector: address(badConn),
                safety:    depSafeWrap2
            });
            vm.prank(gov);
            vm.expectRevert(bytes("CONNECTOR_ASSET_MISMATCH"));
            cg.setStrategy(keccak256("Y"), s);
        }

        // (3) DepositIdentity: connector asset must be base (no wrapper)
        {
            ConversionGateway.StrategyCfg memory s;
            s.kind = ConversionGateway.RouteKind.DepositIdentity;
            MockExternalVaultConnector needsBase = new MockExternalVaultConnector(address(WRAPPED));
            s.deposit = ConversionGateway.DepositCfg({
                wrapper:   address(0),
                connector: address(needsBase),
                safety:    depSafeIdent2
            });
            vm.prank(gov);
            vm.expectRevert(bytes("CONNECTOR_NEEDS_BASE_ASSET"));
            cg.setStrategy(keccak256("Z"), s);
        }
    }

    /* --------------------- onClaim (deposit) --------------------- */

    function test_onClaim_wrap_happy_path() public {
        address user = makeAddr("alice");
        uint256 baseIn = 123e8;

        // CG should already hold base (post-vault claim)
        BASE.mint(address(cg), baseIn);

        // PL is the only caller
        vm.prank(address(pl));
        cg.onClaimWithStrategy(user, baseIn, STRAT_WRAP, "");

        // connector credited ENT (in WRAPPED units)
        assertEq(connWrapped.assetsOf(user), baseIn, "user entitlement in connector");
        // CG ended with zero base & zero wrapped (wrapped was pulled by connector)
        assertEq(BASE.balanceOf(address(cg)), 0);
        assertEq(WRAPPED.balanceOf(address(cg)), 0);
    }

    function test_onClaim_identity_happy_path() public {
        address user = makeAddr("bob");
        uint256 baseIn = 50e8;

        BASE.mint(address(cg), baseIn);

        vm.prank(address(pl));
        cg.onClaimWithStrategy(user, baseIn, STRAT_IDENT, "");

        // connector credited in BASE units
        assertEq(connBase.assetsOf(user), baseIn);
        assertEq(BASE.balanceOf(address(cg)), 0);
    }

    function test_onClaim_access_and_args() public {
        address user = makeAddr("eve");

        // only PL
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector,
                address(this),         // msg.sender here is the test contract
                cg.ROLE_PL()           // role required by onlyPL
            )
        );
        cg.onClaimWithStrategy(user, 1, STRAT_WRAP, "");

        // bad args
        BASE.mint(address(cg), 1);
        vm.prank(address(pl));
        vm.expectRevert(bytes("BAD_ARGS"));
        cg.onClaimWithStrategy(address(0), 1, STRAT_WRAP, "");

        vm.prank(address(pl));
        vm.expectRevert(bytes("BAD_ARGS"));
        cg.onClaimWithStrategy(user, 0, STRAT_WRAP, "");
    }

    /* --------------------- unwindDepositAny --------------------- */

    function _seedWrappedPosition(address user, uint256 baseIn) internal {
        // fund CG with base and do the onClaim (wrap) so the connector holds entitlement
        BASE.mint(address(cg), baseIn);
        vm.prank(address(pl));
        cg.onClaimWithStrategy(user, baseIn, STRAT_WRAP, "");
        assertEq(connWrapped.assetsOf(user), baseIn);
    }

    function test_unwindDepositAny_all_byKeeper_wrap() public {
        address user = makeAddr("alice");
        uint256 baseIn = 200e8;
        _seedWrappedPosition(user, baseIn);

        // keeper can call
        vm.prank(keeper);
        cg.unwindDepositAny(user, STRAT_WRAP, type(uint256).max);

        // PL got called with exact baseOut, entitlement cleared
        assertEq(pl.called(), 1);
        assertEq(pl.lastUser(), user);
        assertEq(pl.lastAssets(), baseIn);
        assertEq(pl.lastStrat(), STRAT_WRAP);
        assertEq(connWrapped.assetsOf(user), 0);

        // CG now holds the base (since wrapper minted base back on unwrap, but CG forwarded to PL)
        // After PL.repayAndRestake(), CG should not retain base
        assertEq(BASE.balanceOf(address(cg)), 0, "CG should not retain base after restake");
    }

    function test_unwindDepositAny_partial_then_zero_reverts() public {
        address user = makeAddr("bob");
        uint256 baseIn = 100e8;
        _seedWrappedPosition(user, baseIn);

        // PL caller is also allowed
        vm.prank(address(pl));
        cg.unwindDepositAny(user, STRAT_WRAP, 40e8);

        assertEq(pl.called(), 1);
        assertEq(pl.lastAssets(), 40e8);
        assertEq(connWrapped.assetsOf(user), 60e8);

        // request 0 -> NOTHING_TO_REDEEM (entitlement still > 0)
        vm.prank(keeper);
        vm.expectRevert(bytes("NOTHING_TO_REDEEM"));
        cg.unwindDepositAny(user, STRAT_WRAP, 0);
    }

    function test_unwindDepositAny_access_control() public {
        address user = makeAddr("carol");
        _seedWrappedPosition(user, 10e8);

        // random address cannot call
        vm.expectRevert(bytes("ONLY KEEPER OR PL"));
        cg.unwindDepositAny(user, STRAT_WRAP, 1);

        // keeper can
        vm.prank(keeper);
        cg.unwindDepositAny(user, STRAT_WRAP, 1);
        assertEq(pl.called(), 1);
    }

    function test_unwindDepositAny_unwrap_slippage_reverts() public {
        address user = makeAddr("dave");
        uint256 baseIn = 90e8;
        _seedWrappedPosition(user, baseIn);

        // redeem 50e8; connector will return 50e8 wrapped back to CG.
        // Make wrapper under-return 49e8 on unwrap → should revert UNWRAP_SLIPPAGE.
        wrapper.setUnwrapNextOut(49e8);

        vm.prank(keeper);
        vm.expectRevert(bytes("UNWRAP_SLIPPAGE"));
        cg.unwindDepositAny(user, STRAT_WRAP, 50e8);
    }

    function test_unwindDepositAny_identity() public {
        address user = makeAddr("erin");
        uint256 baseIn = 33e8;

        // seed identity strategy
        BASE.mint(address(cg), baseIn);
        vm.prank(address(pl));
        cg.onClaimWithStrategy(user, baseIn, STRAT_IDENT, "");
        assertEq(connBase.assetsOf(user), baseIn);

        // keeper withdraw all
        vm.prank(keeper);
        cg.unwindDepositAny(user, STRAT_IDENT, type(uint256).max);

        assertEq(pl.called(), 1);
        assertEq(pl.lastUser(), user);
        assertEq(pl.lastAssets(), baseIn);
        assertEq(pl.lastStrat(), STRAT_IDENT);
        assertEq(connBase.assetsOf(user), 0);

        // After restake, CG should not keep base
        assertEq(BASE.balanceOf(address(cg)), 0);
    }
}


/* ====================== Tests (Borrow path) ====================== */

contract ConversionGatewayBorrowTest is Test {
    // actors
    address public gov    = makeAddr("gov");
    address public keeper = makeAddr("keeper");
    address public pauser = makeAddr("pauser");
    MockPL  public pl;

    // tokens
    MockERC20 public BASE;   // e.g., WBTC (8d)
    MockERC20 public DEBT;   // e.g., USDC (6d)

    // mocks
    MockOracle             public oracle;
    MockBorrowAdapter      public adapter;
    MockExternalVaultConnector public debtConn;

    // SUT
    ConversionGateway public cg;

    // strategy id
    bytes32 constant STRAT_BORROW = keccak256("BORROW_VS_BASE");

    function setUp() public {
        BASE = new MockERC20("Wrapped BTC", "WBTC", 8);
        DEBT = new MockERC20("USD Coin", "USDC", 6);

        oracle  = new MockOracle();

        debtConn= new MockExternalVaultConnector(address(DEBT));
        pl      = new MockPL(address(BASE));

        // prices (1e8): BTC = 50,000; USDC = 1
        oracle.set(address(BASE), 50_000 * 1e8);
        oracle.set(address(DEBT), 1e8);

        cg = new ConversionGateway(gov, keeper, pauser, address(pl), oracle, IERC20(address(BASE)));

        adapter = new MockBorrowAdapter();

        ConversionGateway.BorrowSafety memory bSafe = ConversionGateway.BorrowSafety({
            redeemToleranceBps: 50,      // 0.50% slack when redeeming debt from connector
            withdrawSlippageBps: 50,     // shave 0.50% from pro-rata collateral withdrawal
            maxAprBps:        1500,      // block new borrows if APR > 15%
            minHfBps:         1200,      // block new borrows if HF < 120% (safety margin)
            emergencyMode:    false,
            emergencyRedeemBps: 300,     // allow 3% shortfall in emergency
            emergencyWithdrawBps: 300    // allow 3% extra shave in emergency
        });


        // configure strategy (BorrowVsBase)
        ConversionGateway.StrategyCfg memory s;
        s.kind = ConversionGateway.RouteKind.BorrowVsBase;
        s.borrow = ConversionGateway.BorrowCfg({
            adapter: address(adapter),
            debtAsset: address(DEBT),
            borrowedConnector: address(debtConn),
            maxBorrowBps: 7000,           // 70% LTV cap
            safety: bSafe
            // add other fields here if your BorrowCfg has more
        });

        vm.prank(gov);
        cg.setStrategy(STRAT_BORROW, s);

        // labels
        vm.label(address(cg), "CG");
        vm.label(address(adapter), "Adapter");
        vm.label(address(debtConn), "DebtConnector");
        vm.label(address(pl), "PL");
        vm.label(address(BASE), "BASE");
        vm.label(address(DEBT), "DEBT");
        vm.label(address(oracle), "Oracle");
    }

    function _onClaimBorrow(address user, uint256 baseAmount, uint16 overrideBps, uint256 minBorrowOut) internal {
        // fund CG with base (as if vault just claimed to CG)
        BASE.mint(address(cg), baseAmount);

        // PL calls onClaimWithStrategy
        bytes memory adapterData   = ""; // not used by mock
        bytes memory connectorData = ""; // not used by mock
        bytes memory params = abi.encode(overrideBps, adapterData, connectorData, minBorrowOut);

        vm.prank(address(pl));
        cg.onClaimWithStrategy(user, baseAmount, STRAT_BORROW, params);
    }

    function test_onClaimWithStrategy_happy() public {
        address user = makeAddr("alice");
        uint256 baseIn = 1e8; // 1 BTC
        // 1 BTC @ $50k, maxBorrowBps=70% => borrowUsd= $35,000 => in USDC (6d) = 35,000e6
        uint256 expectedDebt = 35_000e6;

        _onClaimBorrow(user, baseIn, /*overrideBps*/ 0, /*minBorrowOut*/ expectedDebt);

        // connector attributed debt to user (entitlement)
        assertEq(debtConn.assetsOf(user), expectedDebt, "entitlement in borrowed token");
        // adapter saw collateral & debt
        assertEq(adapter.collateralBalance(address(BASE)), baseIn, "coll supplied");
        assertEq(adapter.debtBalance(address(DEBT)), expectedDebt, "venue debt");
    }

    function test_onClaimWithStrategy_override_lower_bps() public {
        address user = makeAddr("bob");
        uint256 baseIn = 2e8; // 2 BTC
        // default (70%): $100k * 70% = $70k -> 70k USDC
        // override 50%:  $100k * 50% = $50k -> 50k USDC
        uint256 expectedDebt = 50_000e6;

        _onClaimBorrow(user, baseIn, /*overrideBps*/ 5000, /*minBorrowOut*/ expectedDebt);

        assertEq(debtConn.assetsOf(user), expectedDebt);
        assertEq(adapter.debtBalance(address(DEBT)), expectedDebt);
    }

    function test_onClaimWithStrategy_minBorrow_guard() public {
        address user = makeAddr("carol");
        uint256 baseIn = 1e8; // 1 BTC
        // Expect 35k USDC, we force minBorrowOut too high -> revert
        bytes memory params = abi.encode(uint16(0), bytes(""), bytes(""), uint256(35_001e6));

        BASE.mint(address(cg), baseIn);

        vm.prank(address(pl));
        vm.expectRevert(bytes("BORROW_MIN"));
        cg.onClaimWithStrategy(user, baseIn, STRAT_BORROW, params);
    }

    /* ---------------- Unwind (repay + withdraw) ---------------- */

    function _seedBorrowPosition(address user, uint256 baseIn) internal returns (uint256 borrowedOut) {
        // baseIn BTC @ $50k, 70% -> borrowUsd
        uint256 borrowUsd = (50_000e8 /*price*/ * baseIn * (10 ** (18 - 8)) ) / 1e8; // CG does this internally
        borrowUsd = (borrowUsd * 7000) / 10_000; // 70%
        // convert to tokens (USDC): $ -> amount; but we rely on CG’s exact math in onClaim
        _onClaimBorrow(user, baseIn, 0, 1); // minBorrowOut=1 to avoid reverts
        borrowedOut = debtConn.assetsOf(user);
    }

    function test_unwindBorrow_all_entitlement() public {
        address user = makeAddr("dan");
        uint256 baseIn = 3e8; // 3 BTC
        uint256 entitlement = _seedBorrowPosition(user, baseIn);

        // keeper redeems all entitlement (type(uint).max), expects proportional collateral withdrawal
        vm.prank(keeper);
        (uint256 collOut, uint256 repaid, uint256 redeemed) =
            cg.unwindBorrow(user, STRAT_BORROW, type(uint256).max, /*minCollateralOut*/ 1, /*adapterData*/ "", /*connectorMinOut*/ entitlement);

        // venue debt reduced
        assertEq(repaid, redeemed, "repay==redeem");
        assertEq(adapter.debtBalance(address(DEBT)), 0, "debt cleared");

        // some collateral withdrawn and restaked via PL
        assertGt(collOut, 0, "withdrew collateral");
        assertEq(pl.called(), 1);
        assertEq(pl.lastUser(), user);
        assertEq(pl.lastAssets(), collOut);
        assertEq(pl.lastStrat(), STRAT_BORROW);

        // connector entitlement cleared
        assertEq(debtConn.assetsOf(user), 0);
    }

    function test_unwindBorrow_partial_clip_and_slippage_guards() public {
        address user = makeAddr("erin");
        uint256 baseIn = 2e8; // 2 BTC
        uint256 ent = _seedBorrowPosition(user, baseIn);

        // ask to redeem more than entitlement -> should clip to entitlement (by CG before redeem)
        uint256 ask = ent + 123;

        // make adapter return slightly *less* collateral than CG pro-rata expects to trigger slippage guard
        adapter.setwithdrawNextOut( 1); // nonsense small; CG wants > minCollateralOut

        vm.prank(keeper);
        vm.expectRevert(bytes("COLLATERAL_SLIPPAGE"));
        cg.unwindBorrow(user, STRAT_BORROW, ask, /*minCollateralOut*/ 2, /*adapterData*/ "", /*connectorMinOut*/ ent);

        // now allow 0 minCollateralOut so it passes even with tiny withdrawal
        vm.prank(keeper);
        (uint256 collOut, uint256 repaid, uint256 redeemed) =
            cg.unwindBorrow(user, STRAT_BORROW, ask, /*minCollateralOut*/ 0, /*adapterData*/ "", /*connectorMinOut*/ ent);

        assertEq(repaid, redeemed);
        assertEq(debtConn.assetsOf(user), 0);
        assertEq(pl.called(), 1);
        assertEq(pl.lastUser(), user);
        assertEq(pl.lastAssets(), collOut);
    }

    function test_unwindBorrow_connector_minOut_guard() public {
        address user = makeAddr("frank");
        uint256 baseIn = 1e8; // 1 BTC
        uint256 ent = _seedBorrowPosition(user, baseIn);

        // ask exact entitlement but set connectorMinOut too high -> connector revert bubbles to CG
        vm.prank(keeper);
        vm.expectRevert(bytes("redeem/min"));
        cg.unwindBorrow(user, STRAT_BORROW, ent, /*minCollateralOut*/ 0, /*adapterData*/ "", /*connectorMinOut*/ ent + 1);
    }

    function test_access_control() public {
        address user = makeAddr("gary");
        uint256 baseIn = 1e8;
        _seedBorrowPosition(user, baseIn);

        // random cannot unwind
        vm.expectRevert();
        cg.unwindBorrow(user, STRAT_BORROW, 1, 0, "", 1);

    }
}
