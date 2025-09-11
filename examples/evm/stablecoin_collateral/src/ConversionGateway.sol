// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./interfaces/IBorrowVenueAdapter.sol";

interface IPL {
    function repayAndRestake(address user, uint256 assets, bytes32 strategy) external;
    function finalizeUnwind(address user, bytes32 strategy) external;
}

interface IWrapper1to1 {
    function base() external view returns (address);
    function wrapped() external view returns (address);
    function wrap(uint256 amount) external returns (uint256 out); // expect out == amount (1:1)
    function unwrap(uint256 amount) external returns (uint256 out); // expect out == amount (1:1)
}

interface IExternalVaultConnector {
    function asset() external view returns (address); // ERC-4626 underlying
    function depositFor(address user, uint256 assets) external returns (uint256 sharesOut);
    function redeemFor(address user, uint256 requestedAssets, uint256 minAssetsOut)
        external
        returns (uint256 assetsOut, uint256 sharesBurned);
    function assetsOf(address user) external view returns (uint256); // entitlement incl. yield
}

interface IPriceOracle {
    function price(address token) external view returns (uint256); // 1e8
}

interface IRateAwareAdapter is IBorrowVenueAdapter {
    function borrowAprBps(address debtAsset) external view returns (uint16);
    function healthFactorBps() external view returns (uint16);
}

/**
 * @title ConversionGateway
 * @notice Middle layer between SatLayer Vault (PositionLocker) and connectors.
 *
 * @dev
 * Purpose:
 *  - Receives SatLayer Vault’s base asset (e.g. WBTC) after async redemption.
 *  - Routes that asset into either:
 *      a) Direct deposit route (identity: base -> borrowed vault directly).
 *      b) Wrap route (BTC -> wBTC -> vault).
 *      c) Borrow-vs-BTC route (post-collateral borrow of borroweds).
 *  - Manages unwind paths in reverse, ensuring borroweds -> base asset -> back to PositionLocker.
 *
 * Flow:
 *  1. `onClaimWithStrategy` is called by PositionLocker after claim.
 *     - CG looks up strategy config (wrapper?, connector?).
 *     - Optionally wraps base to another token (1:1).
 *     - Calls connector.depositFor(user, amount).
 *  2. On unwind:
 *     - Keeper or PositionLocker calls `unwindWrapAny` (or borrow unwind variant).
 *     - CG redeems borroweds from connector, unwraps if needed.
 *     - Swaps/unwraps back to base.
 *     - Calls `repayAndRestake(user, assets, strategy)` on PL.
 *
 * Key Points:
 *  - Holds routing logic + strategy registry.
 *  - Enforces slippage checks (minOut).
 *  - Only PositionLocker and Keeper can call critical flows.
 *  - User never touches this contract directly.
 */
contract ConversionGateway is AccessControl, ReentrancyGuard {
    IERC20 public immutable baseAsset; // SatLayer base token
    address public immutable pl; //  Position Locker
    IPriceOracle public oracle; // prices in 1e8 USD
    bytes32 public constant ROLE_GOVERNANCE = keccak256("ROLE_GOVERNANCE"); // Operator
    bytes32 public constant ROLE_KEEPER = keccak256("ROLE_KEEPER");
    bytes32 public constant ROLE_PL = keccak256("ROLE_POSITION_LOCKER");
    bytes32 public constant ROLE_PAUSER = keccak256("ROLE_PAUSER");

    enum RouteKind {
        DepositIdentity,
        DepositWrap1to1,
        BorrowVsBase
    }

    struct DepositCfg {
        address wrapper; // IWrapper1to1 or address(0) for identity
        address connector; // IExternalVaultConnector that holds per-user "target asset"
        DepositSafety safety;
    }

    struct BorrowCfg {
        address adapter; // IBorrowVenueAdapter (Aave/Morpho/…)
        address debtAsset; // token borrowed (USDC/DAI/WETH/…)
        address borrowedConnector; // IExternalVaultConnector for per-user debtAsset (
        // policy knobs for unwind:
        uint16 maxBorrowBps; // e.g., 7000 = 70% of collateral USD (cap per strategy)
        BorrowSafety safety;
    }

    struct StrategyCfg {
        RouteKind kind;
        DepositCfg deposit; // used when kind is DepositIdentity/DepositWrap1to1
        BorrowCfg borrow; // used when kind is BorrowVsBase
    }

    struct DepositSafety {
        // connector redeem guard (handles 4626 rounding/fees)
        uint16 redeemToleranceBps; // e.g. 25 = 0.25%
        // unwrap guard (only for wrap1to1)
        uint16 unwrapMinOutBps; // min out vs redeemed wrapped (e.g. 9950 = 99.5%)
        // emergency overrides
        bool emergencyMode;
        uint16 emergencyRedeemBps; // wider minOut during emergency
        uint16 emergencyUnwrapBps; // wider unwrap tolerance during emergency
    }

    struct BorrowSafety {
        // connector redeem guard for debt token
        uint16 redeemToleranceBps; // e.g. 50 = 0.5%
        // pro-rata collateral withdrawal shave (rounding / venue quirks)
        uint16 withdrawSlippageBps; // e.g. 50 = 0.5%
        // dynamic ceilings (pre-borrow gate)
        uint256 maxAprBps; // 0 = ignore; otherwise block if APR > maxAprBps
        uint256 minHfBps; // 0 = ignore; otherwise block if HF < minHfBps
        // emergency overrides
        bool emergencyMode;
        uint16 emergencyRedeemBps; // wider during emergency
        uint16 emergencyWithdrawBps; // wider during emergency
    }

    mapping(bytes32 => BorrowSafety) public borrowSafety;

    mapping(bytes32 => StrategyCfg) public strategies;
    mapping(address => bool) public borrowedAllowlist;

    mapping(bytes32 => DepositSafety) public depositEmergency;

    uint256 private constant P8 = 1e8;

    uint16 private constant BPS_MAX = 10_000;

    event StrategySet(
        bytes32 indexed id,
        RouteKind kind,
        // Deposit config
        address depositWrapper,
        address depositConnector,
        // Borrow config
        address borrowAdapter,
        address debtAsset,
        address borrowedConnector
    );

    event DepositedWrap(address indexed user, bytes32 indexed strategy, uint256 baseIn, uint256 wrappedDeposited);
    event UnwoundDeposit(address indexed user, bytes32 indexed strategy, uint256 baseOut);
    event BorrowSuppliedAndDrawn(address indexed user, bytes32 indexed strategy, uint256 baseIn, uint256 debtOut);
    event UnwoundBorrow(
        address indexed user,
        bytes32 indexed strategy,
        uint256 debtRedeemed,
        uint256 debtRepaid,
        uint256 collateralRestaked
    );

    event StrategyBorrowEmergencySet(bytes32 indexed strategy, bool on);
    event StrategyDepositEmergencySet(bytes32 indexed strategy, bool on);
    event StrategyBorrowSafetySet(bytes32 indexed strategy, uint16 redeemToleranceBps, uint16 withdrawSlippageBps);
    event StrategyDepositSafetySet(bytes32 indexed strategy, uint16 redeemToleranceBps, uint16 unwrapMinOutBps);

    constructor(address governance, address keeper, address pauser, address _pl, IPriceOracle _oracle, IERC20 _base) {
        require(address(_base) != address(0) && _pl != address(0), "ZERO_ADDR");
        baseAsset = _base;
        pl = _pl;
        oracle = _oracle;
        _grantRole(ROLE_GOVERNANCE, governance);
        _grantRole(ROLE_KEEPER, keeper);
        _grantRole(ROLE_PAUSER, pauser);
        _grantRole(ROLE_PL, pl);
    }

    function grantKeeper(address k) external onlyRole(ROLE_GOVERNANCE) {
        _grantRole(ROLE_KEEPER, k);
    }

    function setPauser(address p) external onlyRole(ROLE_GOVERNANCE) {
        _grantRole(ROLE_PAUSER, p);
    }

    // Flip emergency latch + optionally pause new borrows
    function setBorrowEmergency(bytes32 strat, bool on) external onlyRole(ROLE_GOVERNANCE) {
        strategies[strat].borrow.safety.emergencyMode = on;
        emit StrategyBorrowEmergencySet(strat, on);
    }

    function setDepositEmergency(bytes32 strat, bool on) external onlyRole(ROLE_GOVERNANCE) {
        strategies[strat].deposit.safety.emergencyMode = on;
        emit StrategyDepositEmergencySet(strat, on);
    }

    // Update borrow safety knobs
    function setBorrowSafety(
        bytes32 strat,
        uint16 redeemToleranceBps,
        uint16 withdrawSlippageBps,
        uint256 maxAprBps,
        uint256 minHfBps,
        uint16 emergencyRedeemBps,
        uint16 emergencyWithdrawBps
    ) external onlyRole(ROLE_GOVERNANCE) {
        BorrowSafety storage S = strategies[strat].borrow.safety;
        S.redeemToleranceBps = redeemToleranceBps;
        S.withdrawSlippageBps = withdrawSlippageBps;
        S.maxAprBps = maxAprBps;
        S.minHfBps = minHfBps;
        S.emergencyRedeemBps = emergencyRedeemBps;
        S.emergencyWithdrawBps = emergencyWithdrawBps;
        emit StrategyBorrowSafetySet(strat, redeemToleranceBps, withdrawSlippageBps);
    }

    // Update deposit safety knobs
    function setDepositSafety(
        bytes32 strat,
        uint16 redeemToleranceBps,
        uint16 unwrapMinOutBps,
        uint16 emergencyRedeemBps,
        uint16 emergencyUnwrapBps
    ) external onlyRole(ROLE_GOVERNANCE) {
        DepositSafety storage S = strategies[strat].deposit.safety;
        S.redeemToleranceBps = redeemToleranceBps;
        S.unwrapMinOutBps = unwrapMinOutBps;
        S.emergencyRedeemBps = emergencyRedeemBps;
        S.emergencyUnwrapBps = emergencyUnwrapBps;
        emit StrategyDepositSafetySet(strat, redeemToleranceBps, unwrapMinOutBps);
    }

    function setOracle(IPriceOracle _oracle) external onlyRole(ROLE_GOVERNANCE) {
        oracle = _oracle;
    }

    function setStrategy(bytes32 id, StrategyCfg calldata s) external onlyRole(ROLE_GOVERNANCE) {
        if (s.kind == RouteKind.DepositIdentity) {
            // identity: no wrapper, connector must accept base
            require(s.deposit.wrapper == address(0), "IDENT_NO_WRAPPER");
            require(s.deposit.connector != address(0), "IDENT_NEEDS_CONNECTOR");
            require(
                IExternalVaultConnector(s.deposit.connector).asset() == address(baseAsset), "CONNECTOR_NEEDS_BASE_ASSET"
            );
            // safety for deposit path
            _validateDepositSafety(s.deposit.safety);

            // borrow cfg should be inert in this kind
            require(
                s.borrow.adapter == address(0) && s.borrow.debtAsset == address(0)
                    && s.borrow.borrowedConnector == address(0),
                "BORROW_CFG_NOT_EMPTY"
            );
        } else if (s.kind == RouteKind.DepositWrap1to1) {
            // wrap path: wrapper must be set and 1:1 over the same base; connector must accept wrapped
            require(s.deposit.wrapper != address(0), "WRAP_NEEDS_WRAPPER");
            require(s.deposit.connector != address(0), "WRAP_NEEDS_CONNECTOR");
            require(IWrapper1to1(s.deposit.wrapper).base() == address(baseAsset), "WRAP_BASE_MISMATCH");
            require(
                IExternalVaultConnector(s.deposit.connector).asset() == IWrapper1to1(s.deposit.wrapper).wrapped(),
                "CONNECTOR_ASSET_MISMATCH"
            );
            _validateDepositSafety(s.deposit.safety);

            // borrow cfg should be inert in this kind
            require(
                s.borrow.adapter == address(0) && s.borrow.debtAsset == address(0)
                    && s.borrow.borrowedConnector == address(0),
                "BORROW_CFG_NOT_EMPTY"
            );
        } else if (s.kind == RouteKind.BorrowVsBase) {
            // borrow path: adapter+debtAsset required
            require(s.borrow.adapter != address(0), "BORROW_NEEDS_ADAPTER");
            require(s.borrow.debtAsset != address(0), "BORROW_NEEDS_DEBT_ASSET");
            require(s.borrow.borrowedConnector != address(0), "BORROW_NEEDS_CONNECTOR");
            require(
                IExternalVaultConnector(s.borrow.borrowedConnector).asset() == s.borrow.debtAsset,
                "CONNECTOR_borrowed_MISMATCH"
            );

            _checkBps(s.borrow.maxBorrowBps);

            // safety for borrow path
            _validateBorrowSafety(s.borrow.safety);

            // deposit cfg should be inert in this kind
            require(s.deposit.wrapper == address(0) && s.deposit.connector == address(0), "DEPOSIT_CFG_NOT_EMPTY");
        } else {
            revert("UNKNOWN_KIND");
        }

        strategies[id] = s;

        emit StrategySet(
            id,
            s.kind,
            s.deposit.wrapper,
            s.deposit.connector,
            s.borrow.adapter,
            s.borrow.debtAsset,
            s.borrow.borrowedConnector
        );
    }

    /// @notice Router called by PL after claim; branches by strategy kind.
    /// @param user   end user
    /// @param baseIn base asset amount the vault already paid to CG
    /// @param strat_id  strategy id
    /// @param params optional strategy-specific bytes:
    ///   - DepositIdentity / DepositWrap1to1: usually empty
    ///   - BorrowVsBase: abi.encode(uint16 borrowBpsOverride, bytes adapterData, bytes connectorData, uint256 minBorrowOut)
    function onClaimWithStrategy(address user, uint256 baseIn, bytes32 strat_id, bytes calldata params)
        external
        nonReentrant
        onlyRole(ROLE_PL)
    {
        StrategyCfg memory S = strategies[strat_id];
        if (S.kind == RouteKind.DepositIdentity || S.kind == RouteKind.DepositWrap1to1) {
            // params typically empty; ignored
            _onClaimWithStrategyDeposit(user, baseIn, strat_id);
        } else if (S.kind == RouteKind.BorrowVsBase) {
            _onClaimWithStrategyBorrow(user, baseIn, strat_id, params);
        } else {
            revert("UNKNOWN_STRAT_KIND");
        }
    }

    /* ----------------------------------------------------------------
    * onClaimWithStrategyDeposit
    *  - Called by PL after vault has paid base asset to this CG
    *  - For DepositIdentity: deposit base directly into connector for the user
    *  - For DepositWrap1to1: wrap base 1:1, then deposit wrapped into connector
    * ---------------------------------------------------------------- */
    function _onClaimWithStrategyDeposit(address user, uint256 baseIn, bytes32 strategy) internal {
        require(user != address(0) && baseIn > 0, "BAD_ARGS");
        StrategyCfg memory cfg = strategies[strategy];
        require(cfg.kind == RouteKind.DepositIdentity || cfg.kind == RouteKind.DepositWrap1to1, "NOT_DEPOSIT_KIND");

        address tokenToDeposit = address(baseAsset);
        uint256 amt = baseIn;

        // If wrapping, enforce 1:1 wrapper over the same base
        if (cfg.kind == RouteKind.DepositWrap1to1) {
            address w = cfg.deposit.wrapper;
            require(w != address(0), "WRAP_NOT_SET");
            require(IWrapper1to1(w).base() == address(baseAsset), "WRAP_BASE_MISMATCH");
            tokenToDeposit = IWrapper1to1(w).wrapped();

            // Wrap base -> wrapped (1:1)
            IERC20(address(baseAsset)).approve(w, baseIn);
            uint256 out = IWrapper1to1(w).wrap(baseIn);
            require(out == baseIn, "WRAP_NOT_1_TO_1");
            amt = out;
        } else {
            // Identity path: connector must accept base directly
            tokenToDeposit = address(baseAsset);
        }

        // Deposit into external ERC-4626-like connector, attributing to user
        require(cfg.deposit.connector != address(0), "CONNECTOR_ZERO");
        require(IExternalVaultConnector(cfg.deposit.connector).asset() == tokenToDeposit, "CONN_ASSET_MISMATCH");
        IERC20(tokenToDeposit).approve(cfg.deposit.connector, amt);
        IExternalVaultConnector(cfg.deposit.connector).depositFor(user, amt);

        emit DepositedWrap(user, strategy, baseIn, amt);
    }

    /* ----------------------------------------------------------------
    * unwindDepositAny
    *  - Redeem user's position from connector
    *  - If wrapped, unwrap 1:1 back to base
    *  - Restake base to PL for the user (reduces their PL debt)
    * ---------------------------------------------------------------- */

    function unwindDepositAny(
        address user,
        bytes32 strategy,
        uint256 requestedBaseOrWrapped // pass type(uint256).max for "all"
    ) external nonReentrant onlyKeeperOrPL {
        StrategyCfg memory cfg = strategies[strategy];
        DepositSafety memory safe = cfg.deposit.safety;

        require(cfg.kind == RouteKind.DepositIdentity || cfg.kind == RouteKind.DepositWrap1to1, "NOT_DEPOSIT_KIND");
        require(cfg.deposit.connector != address(0), "CONNECTOR_ZERO");

        // Determine entitlement in connector units (base on Identity; wrapped on Wrap1to1)
        uint256 entitlement = IExternalVaultConnector(cfg.deposit.connector).assetsOf(user);
        require(entitlement > 0, "NO_ENTITLEMENT");

        uint256 toRedeem = (requestedBaseOrWrapped == type(uint256).max)
            ? entitlement
            : (requestedBaseOrWrapped <= entitlement ? requestedBaseOrWrapped : entitlement);
        require(toRedeem > 0, "NOTHING_TO_REDEEM");

        // Redeem from connector -> choose tolerance based on emergency mode
        uint16 redeemBps = safe.emergencyMode ? safe.emergencyRedeemBps : safe.redeemToleranceBps;
        uint256 minOut = (toRedeem * (10_000 - redeemBps)) / 10_000;

        (uint256 outWrappedOrBase,) = IExternalVaultConnector(cfg.deposit.connector).redeemFor(user, toRedeem, minOut);
        require(outWrappedOrBase >= minOut, "REDEEM_SHORTFALL");

        // If wrapped path, unwrap back to base with its own tolerance (emergency-aware)
        uint256 baseOut = outWrappedOrBase;
        if (cfg.kind == RouteKind.DepositWrap1to1) {
            address w = cfg.deposit.wrapper;
            require(w != address(0), "WRAP_NOT_SET");

            address wrapped = IWrapper1to1(w).wrapped();
            IERC20(wrapped).approve(w, outWrappedOrBase);

            //Unwrap & enforce min-out under normal/emergency settings
            // unwrap guard: unwrapMinOutBps is the *minimum* fraction of wrapped required as base
            uint16 minBps = safe.emergencyMode ? safe.emergencyUnwrapBps : safe.unwrapMinOutBps;
            // e.g. unwrapMinOutBps = 10_000 => require 1:1; 9_950 => allow up to 0.5% loss
            uint256 minBaseOut = (outWrappedOrBase * minBps) / 10_000;

            uint256 out = IWrapper1to1(w).unwrap(outWrappedOrBase);
            require(out >= minBaseOut, "UNWRAP_SLIPPAGE");
            baseOut = out;
        }

        // Restake to PL on behalf of user
        IERC20(address(baseAsset)).approve(pl, baseOut);
        IPL(pl).repayAndRestake(user, baseOut, strategy);
        if (IExternalVaultConnector(cfg.deposit.connector).assetsOf(user) == 0) {
            IPL(pl).finalizeUnwind(user, strategy);
        }

        emit UnwoundDeposit(user, strategy, baseOut);
    }

    /* ===========================================================
     * onClaimWithStrategyBorrow
     *  - Called by PL right after the SatLayer vault *claimed* base asset to this CG
     *  - We supply base as collateral to the venue adapter and borrow `debtAsset`
     *  - If borrowedConnector != 0x0, we deposit borrowed tokens for the user to track entitlement+yield
     * =========================================================== */

    /**
     * @param user     Beneficiary user
     * @param baseIn   Base asset amount (e.g., WBTC) already held by this CG (sent by vault on claim)
     * @param strategy Strategy id; must be configured as BorrowVsBase
     * @param params   abi.encode(uint16 borrowBpsOverride, bytes adapterData, bytes connectorData, uint256 minBorrowOut)
     */
    function _onClaimWithStrategyBorrow(address user, uint256 baseIn, bytes32 strategy, bytes calldata params)
        internal
    {
        require(user != address(0) && baseIn > 0, "BAD_ARGS");

        // Use STORAGE refs to avoid copying large structs to memory
        StrategyCfg storage S = strategies[strategy];
        require(S.kind == RouteKind.BorrowVsBase, "STRAT_KIND");
        IBorrowVenueAdapter adapter = IBorrowVenueAdapter(S.borrow.adapter);

        (bool ok, uint8 reason) = violatesCeilings(S.borrow.safety, adapter, S.borrow.debtAsset);
        require(ok, "BORROW_BLOCKED");

        BorrowCfg storage B = S.borrow;
        require(B.adapter != address(0) && B.debtAsset != address(0), "CFG_MISSING");

        // 1) supply collateral to venue (decode once just to fetch adapterData; keep scope tight)
        {
            (, bytes memory adapterData,,) = abi.decode(params, (uint16, bytes, bytes, uint256));
            _supplyCollateral(B, baseIn, adapterData); // derives adapter inside; keeps callsite lean
        }

        // 2) compute borrow amount (USD-capped) and convert to debt units
        uint256 debtOut;
        {
            // Decode only the two values needed for computation; keep them in a small scope
            (uint16 overrideBps,,, uint256 minBorrowOut) = abi.decode(params, (uint16, bytes, bytes, uint256));
            debtOut = _computeDebtOut(B, baseIn, overrideBps, minBorrowOut);
        }

        // 3 & 4) borrow, then route borrowed tokens (decode routing bytes in this isolated scope)
        {
            (, bytes memory adapterData, bytes memory connectorData,) =
                abi.decode(params, (uint16, bytes, bytes, uint256));
            _drawAndRouteBorrow(B, user, debtOut, adapterData, connectorData);
        }

        emit BorrowSuppliedAndDrawn(user, strategy, baseIn, debtOut);
    }

    // Helper that checks live ceilings (best-effort; skip if adapter doesn’t expose)

    function violatesCeilings(BorrowSafety memory S, IBorrowVenueAdapter adapter, address debtAsset)
        public
        view
        returns (bool ok, uint8 reason)
    {
        if (S.emergencyMode) return (false, 1);

        (bool hasApr, uint256 aprBps, bool haveHf, uint256 hfBps) = adapter.getRiskSignals(debtAsset);

        if (S.maxAprBps > 0 && hasApr && aprBps > S.maxAprBps) {
            return (false, 2);
        }
        if (S.minHfBps > 0 && haveHf && hfBps < S.minHfBps) {
            return (false, 3);
        }
        return (true, 0);
    }

    /* ===========================================================
     * unwindBorrow
     *  - Redeem user's borrowed tokens from borrowedConnector
     *  - Repay venue debt
     *  - Withdraw proportional base collateral
     *  - Send base back to PL.repayAndRestake(user)
     * =========================================================== */

    /**
     * @param user              Position owner
     * @param strategy          Strategy id configured as BorrowVsBase
     * @param requestedDebtIn   How much debtAsset to redeem from connector (use type(uint256).max for "all entitlement")
     * @param minCollateralOut  Minimum base required to proceed (slippage guard)
     * @param adapterData       Venue-specific bytes
     * @param connectorMinOut   Min debtAsset the connector must return (usually = requestedDebtIn, or tolerance-adjusted)
     */
    function unwindBorrow(
        address user,
        bytes32 strategy,
        uint256 requestedDebtIn,
        uint256 minCollateralOut,
        bytes calldata adapterData,
        uint256 connectorMinOut
    ) external nonReentrant onlyKeeperOrPL returns (uint256 collateralOut, uint256 repaidDebt, uint256 redeemedDebt) {
        // Use STORAGE refs to avoid copying big structs to memory
        StrategyCfg storage S = strategies[strategy];
        require(S.kind == RouteKind.BorrowVsBase, "STRAT_KIND");

        BorrowCfg storage B = S.borrow;
        require(
            B.adapter != address(0) && B.debtAsset != address(0) && B.borrowedConnector != address(0), "CFG_MISSING"
        );

        // Keep scopes tight so earlier locals die before later ones
        {
            redeemedDebt = _redeemFromConnector(user, B, requestedDebtIn, connectorMinOut);
        }

        uint256 beforeDebt;
        {
            IBorrowVenueAdapter adapter_ = IBorrowVenueAdapter(B.adapter);
            IERC20 debt = IERC20(B.debtAsset);
            (repaidDebt, beforeDebt) = _repayVenueDebt(adapter_, debt, redeemedDebt, adapterData);
        }

        {
            // compute + withdraw collateral with a slimmer param list
            collateralOut = _withdrawProportionalCollateral(
                B, // storage ref; we recreate adapter_ inside
                adapterData,
                beforeDebt,
                repaidDebt,
                minCollateralOut
            );
        }

        // Restake in its own tiny scope to keep locals minimal
        {
            IERC20(address(baseAsset)).approve(pl, collateralOut);
            IPL(pl).repayAndRestake(user, collateralOut, strategy);
        }

        if (
            IExternalVaultConnector(B.borrowedConnector).assetsOf(user) == 0
                && IBorrowVenueAdapter(B.adapter).debtBalance(B.debtAsset) == 0
        ) {
            IPL(pl).finalizeUnwind(user, strategy);
        }

        emit UnwoundBorrow(user, strategy, redeemedDebt, repaidDebt, collateralOut);
    }

    /* ========================= internals ========================= */

    function _supplyCollateral(BorrowCfg storage B, uint256 baseIn, bytes memory adapterData) internal {
        baseAsset.approve(B.adapter, baseIn);
        IBorrowVenueAdapter(B.adapter).supplyCollateral(address(baseAsset), baseIn, adapterData);
    }

    function _computeDebtOut(BorrowCfg storage B, uint256 baseIn, uint16 overrideBps, uint256 minBorrowOut)
        internal
        returns (uint256 debtOut)
    {
        uint16 borrowBps = (overrideBps > 0 && overrideBps < B.maxBorrowBps) ? overrideBps : B.maxBorrowBps;

        uint256 baseUsd = _tokenUsd(address(baseAsset), baseIn);
        uint256 borrowUsd = (baseUsd * borrowBps) / 10_000;

        debtOut = _usdToToken(B.debtAsset, borrowUsd);
        require(debtOut >= minBorrowOut, "BORROW_MIN");
    }

    function _drawAndRouteBorrow(
        BorrowCfg storage B,
        address user,
        uint256 debtOut,
        bytes memory adapterData,
        bytes memory /* connectorData */ // kept for future use; currently unused
    ) internal {
        if (debtOut == 0) return;

        IBorrowVenueAdapter(B.adapter).borrow(B.debtAsset, debtOut, adapterData);

        if (B.borrowedConnector != address(0)) {
            IERC20(B.debtAsset).approve(B.borrowedConnector, debtOut);
            IExternalVaultConnector(B.borrowedConnector).depositFor(user, debtOut);
        } else {
            revert("NO_BORROW_DEST");
        }
    }

    function _checkBps(uint16 v) private pure {
        require(v <= BPS_MAX, "BPS>100%");
    }

    function _validateDepositSafety(DepositSafety memory s) private pure {
        _checkBps(s.redeemToleranceBps); // tolerance used on connector redeem
        _checkBps(s.unwrapMinOutBps); // min-out for unwrap (e.g., 9_950 = 99.5%)
        _checkBps(s.emergencyRedeemBps); // looser redeem floor in emergency (<= 10_000)
        _checkBps(s.emergencyUnwrapBps); // looser unwrap floor in emergency
    }

    function _validateBorrowSafety(BorrowSafety memory s) private pure {
        _checkBps(s.redeemToleranceBps); // tolerance used on connector redeem
        _checkBps(s.withdrawSlippageBps); // shave on pro-rata withdraw from venue
        _checkBps(s.emergencyRedeemBps); // looser redeem floor in emergency
        _checkBps(s.emergencyWithdrawBps); // looser withdraw floor in emergency
    }

    function _tokenUsd(address token, uint256 amount) internal view returns (uint256 usd1e18) {
        if (amount == 0) return 0;
        uint256 p = oracle.price(token); // 1e8
        uint8 dec = IERC20Metadata(token).decimals();
        // usd(1e18) = amount * price(1e8) * 10^(18-dec) / 1e8
        return (amount * p * (10 ** (18 - dec))) / P8;
    }

    function _usdToToken(address token, uint256 usd1e18) internal view returns (uint256 amount) {
        if (usd1e18 == 0) return 0;
        uint256 p = oracle.price(token); // 1e8
        uint8 dec = IERC20Metadata(token).decimals();
        // amount = usd(1e18) * 1e8 / price / 10^(18-dec)
        uint256 t = (usd1e18 * P8) / p;
        return t / (10 ** (18 - dec));
    }

    function _redeemFromConnector(address user, BorrowCfg storage B, uint256 requestedDebtIn, uint256 connectorMinOut)
        internal
        returns (uint256 redeemedDebt)
    {
        // Safety knobs live inside the borrow cfg
        BorrowSafety memory safe = B.safety;

        require(B.borrowedConnector != address(0), "CONNECTOR_ZERO");

        uint256 entitlement = IExternalVaultConnector(B.borrowedConnector).assetsOf(user);
        require(entitlement > 0, "NO_ENTITLEMENT");

        uint256 toRedeem = requestedDebtIn == type(uint256).max
            ? entitlement
            : (requestedDebtIn <= entitlement ? requestedDebtIn : entitlement);
        require(toRedeem > 0, "ZERO_REQUEST");

        // Policy minOut (tolerance-style)
        uint16 tolBps = safe.emergencyMode ? safe.emergencyRedeemBps : safe.redeemToleranceBps;
        uint256 policyMinOut = (toRedeem * (10_000 - tolBps)) / 10_000;

        // Enforce the stricter requirement: caller override wins if higher
        uint256 minOut = connectorMinOut > policyMinOut ? connectorMinOut : policyMinOut;

        (uint256 debtIn,) = IExternalVaultConnector(B.borrowedConnector).redeemFor(user, toRedeem, minOut);
        require(debtIn >= minOut, "REDEEM_SHORTFALL");

        return debtIn;
    }

    function _repayVenueDebt(IBorrowVenueAdapter adapter_, IERC20 debt, uint256 debtIn, bytes calldata adapterData)
        internal
        returns (uint256 repaidDebt, uint256 beforeDebt)
    {
        beforeDebt = adapter_.debtBalance(address(debt));
        debt.approve(address(adapter_), debtIn);
        repaidDebt = adapter_.repay(address(debt), debtIn, adapterData);
        require(repaidDebt > 0, "REPAY_ZERO");

        uint256 afterDebt = adapter_.debtBalance(address(debt));
        require(beforeDebt >= afterDebt && (beforeDebt - afterDebt) >= repaidDebt, "DEBT_MISMATCH");
    }

    function _withdrawProportionalCollateral(
        BorrowCfg storage B,
        bytes calldata adapterData,
        uint256 beforeDebt,
        uint256 repaidDebt,
        uint256 minCollateralOut
    ) internal returns (uint256 collateralOut) {
        IBorrowVenueAdapter adapter_ = IBorrowVenueAdapter(B.adapter);

        uint256 collBal = adapter_.collateralBalance(address(baseAsset));
        uint256 proRata = beforeDebt == 0 ? 0 : (collBal * repaidDebt) / beforeDebt;

        // Apply configured shave to absorb venue rounding / index drift
        uint16 shaveBps = B.safety.withdrawSlippageBps; // from embedded BorrowSafety
        if (shaveBps > 0) {
            proRata = proRata - ((proRata * shaveBps) / 10_000);
        }
        require(proRata > 0, "NOTHING_TO_WITHDRAW");

        collateralOut = adapter_.withdrawCollateral(address(baseAsset), proRata, adapterData);
        require(collateralOut >= minCollateralOut, "COLLATERAL_SLIPPAGE");
    }

    /* ========== Modifiers for callers ========== */

    modifier onlyKeeperOrPL() {
        require(hasRole(ROLE_KEEPER, msg.sender) || hasRole(ROLE_PL, msg.sender), "ONLY KEEPER OR PL");
        _;
    }
}
