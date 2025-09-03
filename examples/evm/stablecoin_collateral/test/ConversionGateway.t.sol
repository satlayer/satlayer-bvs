// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";

import "./MockERC20.sol";

import "../src/ConversionGateway.sol";

contract MockWrapper1to1 is IWrapper1to1 {
    MockERC20 public immutable _base;
    MockERC20 public immutable _wrapped;

    uint256 public unwrapNextOut;

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

    function setUnwrapNextOut(uint256 v) external {
        unwrapNextOut = v;
    }

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

    constructor(address asset_) {
        _asset = asset_;
    }

    function asset() external view override returns (address) {
        return _asset;
    }

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

contract MockPL {
    address public lastUser;
    uint256 public lastAssets;
    uint256 public called;

    function repayAndRestake(address user, uint256 assets, bytes32 strategy) external {
        lastUser = user;
        lastAssets = assets;
        called++;
    }
}

contract ConversionGatewayTest is Test {
    // actors
    address public gov = makeAddr("gov");
    address public keeper = makeAddr("keeper");
    address public pauser = makeAddr("pauser");
    MockPL public pl; // gets ROLE_PL

    // tokens
    MockERC20 public BASE; // e.g. WBTC
    MockERC20 public WRAPPED; // 1:1 wrapper token
    MockERC20 public OTHER; // random ERC20 to test mismatches

    // system under test
    ConversionGateway public cg;

    // mocks
    MockWrapper1to1 public wrapper;
    MockExternalVaultConnector public connWrapped; // expects WRAPPED
    MockExternalVaultConnector public connBase; // expects BASE

    bytes32 constant STRAT_WRAP = keccak256("WRAP");
    bytes32 constant STRAT_IDENT = keccak256("IDENTITY");

    function setUp() public {
        BASE = new MockERC20("Wrapped BTC", "WBTC", 8);
        WRAPPED = new MockERC20("wWBTC", "wWBTC", 8);
        OTHER = new MockERC20("OTHER", "OTH", 18);

        pl = new MockPL();

        cg = new ConversionGateway(gov, keeper, pauser, address(pl), IERC20(address(BASE)));

        wrapper = new MockWrapper1to1(BASE, WRAPPED);
        connWrapped = new MockExternalVaultConnector(address(WRAPPED));
        connBase = new MockExternalVaultConnector(address(BASE));

        // set a wrapper strategy (wrap base -> wrapped -> deposit)
        vm.prank(gov);
        cg.setStrategyWrap(STRAT_WRAP, address(wrapper), address(connWrapped));

        // set an identity strategy (no wrapper; deposit base directly)
        vm.prank(gov);
        cg.setStrategyWrap(STRAT_IDENT, address(0), address(connBase));

        // labels
        vm.label(address(cg), "CG");
        vm.label(address(wrapper), "Wrapper");
        vm.label(address(connWrapped), "ConnWrapped");
        vm.label(address(connBase), "ConnBase");
        vm.label(address(pl), "PL");
        vm.label(address(BASE), "BASE");
        vm.label(address(WRAPPED), "WRAPPED");
    }

    /* --------------------- setStrategyWrap checks --------------------- */

    function test_setStrategyWrap_validations() public {
        // connector = zero
        vm.prank(gov);
        vm.expectRevert(bytes("CONNECTOR_ZERO"));
        cg.setStrategyWrap(keccak256("X"), address(wrapper), address(0));

        // wrapper base mismatch
        MockWrapper1to1 badWrap = new MockWrapper1to1(OTHER, WRAPPED);
        vm.prank(gov);
        vm.expectRevert(bytes("WRAP_BASE_MISMATCH"));
        cg.setStrategyWrap(keccak256("Y"), address(badWrap), address(connWrapped));

        // wrapper set but connector asset != wrapper.wrapped
        MockExternalVaultConnector badConn = new MockExternalVaultConnector(address(BASE));
        vm.prank(gov);
        vm.expectRevert(bytes("CONNECTOR_ASSET_MISMATCH"));
        cg.setStrategyWrap(keccak256("Z"), address(wrapper), address(badConn));

        // identity path requires connector asset == base
        MockExternalVaultConnector connNeedsBase = new MockExternalVaultConnector(address(WRAPPED));
        vm.prank(gov);
        vm.expectRevert(bytes("CONNECTOR_NEEDS_BASE_ASSET"));
        cg.setStrategyWrap(keccak256("I"), address(0), address(connNeedsBase));
    }

    function test_onClaim_wrap_happy_path() public {
        address user = makeAddr("alice");
        uint256 baseIn = 123e8;

        // CG should already hold base (post-vault claim)
        BASE.mint(address(cg), baseIn);

        // PL is the only caller
        vm.prank(address(pl));
        cg.onClaimWithStrategy(user, baseIn, STRAT_WRAP);

        // connector credited ENT
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
        cg.onClaimWithStrategy(user, baseIn, STRAT_IDENT);

        assertEq(connBase.assetsOf(user), baseIn);
        assertEq(BASE.balanceOf(address(cg)), 0);
    }

    function test_onClaim_access_and_args() public {
        address user = makeAddr("eve");

        // only PL
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector,
                address(this), // msg.sender here is the test contract
                cg.ROLE_PL() // role required by onlyRole(ROLE_PL)
            )
        );

        cg.onClaimWithStrategy(user, 1, STRAT_WRAP);

        // bad args
        BASE.mint(address(cg), 1);
        vm.prank(address(pl));
        vm.expectRevert(bytes("BAD_ARGS"));
        cg.onClaimWithStrategy(address(0), 1, STRAT_WRAP);

        vm.prank(address(pl));
        vm.expectRevert(bytes("BAD_ARGS"));
        cg.onClaimWithStrategy(user, 0, STRAT_WRAP);
    }

    function _seedWrappedPosition(address user, uint256 baseIn) internal {
        // fund CG with base and do the onClaim (wrap) so the connector holds entitlement
        BASE.mint(address(cg), baseIn);
        vm.prank(address(pl));
        cg.onClaimWithStrategy(user, baseIn, STRAT_WRAP);
        assertEq(connWrapped.assetsOf(user), baseIn);
    }

    function test_unwindWrapAny_all_byKeeper() public {
        address user = makeAddr("alice");
        uint256 baseIn = 200e8;
        _seedWrappedPosition(user, baseIn);

        // keeper can call
        vm.prank(keeper);
        cg.unwindWrapAny(user, STRAT_WRAP, type(uint256).max);

        // PL got called with exact baseOut, entitlement cleared
        assertEq(pl.called(), 1);
        assertEq(pl.lastUser(), user);
        assertEq(pl.lastAssets(), baseIn);
        assertEq(connWrapped.assetsOf(user), 0);

        // CG now holds the base (since wrapper minted base back on unwrap)
        assertEq(BASE.balanceOf(address(cg)), baseIn);
    }

    function test_unwindWrapAny_partial_then_zero_reverts() public {
        address user = makeAddr("bob");
        uint256 baseIn = 100e8;
        _seedWrappedPosition(user, baseIn);

        // PL caller is also allowed
        vm.prank(address(pl));
        cg.unwindWrapAny(user, STRAT_WRAP, 40e8);

        assertEq(pl.called(), 1);
        assertEq(pl.lastAssets(), 40e8);
        assertEq(connWrapped.assetsOf(user), 60e8);

        // request 0 -> NOTHING_TO_REDEEM (entitlement still > 0)
        vm.prank(keeper);
        vm.expectRevert(bytes("NOTHING_TO_REDEEM"));
        cg.unwindWrapAny(user, STRAT_WRAP, 0);
    }

    function test_unwindWrapAny_access_control() public {
        address user = makeAddr("carol");
        _seedWrappedPosition(user, 10e8);

        // random address cannot call
        vm.expectRevert(bytes("ONLY KEEPER OR PL"));
        cg.unwindWrapAny(user, STRAT_WRAP, 1);

        // keeper can
        vm.prank(keeper);
        cg.unwindWrapAny(user, STRAT_WRAP, 1);
        assertEq(pl.called(), 1);
    }

    function test_unwindWrapAny_unwrap_slippage_reverts() public {
        address user = makeAddr("dave");
        uint256 baseIn = 90e8;
        _seedWrappedPosition(user, baseIn);

        // redeem 50e8; connector will return 50e8 wrapped back to CG.
        // Make wrapper under-return 49e8 on unwrap → should revert UNWRAP_SLIPPAGE.
        wrapper.setUnwrapNextOut(49e8);

        vm.prank(keeper);
        vm.expectRevert(bytes("UNWRAP_SLIPPAGE"));
        cg.unwindWrapAny(user, STRAT_WRAP, 50e8);
    }

    function test_unwindWrapAny_identity() public {
        address user = makeAddr("erin");
        uint256 baseIn = 33e8;

        // seed identity strategy
        BASE.mint(address(cg), baseIn);
        vm.prank(address(pl));
        cg.onClaimWithStrategy(user, baseIn, STRAT_IDENT);
        assertEq(connBase.assetsOf(user), baseIn);

        // keeper withdraw all
        vm.prank(keeper);
        cg.unwindWrapAny(user, STRAT_IDENT, type(uint256).max);

        assertEq(pl.called(), 1);
        assertEq(pl.lastUser(), user);
        assertEq(pl.lastAssets(), baseIn);
        assertEq(connBase.assetsOf(user), 0);

        // CG now holds base returned directly by connector (no unwrap step)
        assertEq(BASE.balanceOf(address(cg)), baseIn);
    }
}
