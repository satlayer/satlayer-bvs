// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {IBorrowVenueAdapter} from "./Interfaces/IBorrowVenueAdapter.sol";

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
 *      c) Borrow-vs-BTC route (post-collateral borrow of borroweds). (future)
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
        address borrowedConnector; // IExternalVaultConnector for per-user debtAsset (optional if you don’t park)
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

    // = 2; //Tolerance in case vault have  redeem fee
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

    event BorrowOpened(
        address indexed user, bytes32 indexed strategy, uint256 baseIn, uint16 maxBorrowBps, uint256 borrowedBorrowed
    );
    event BorrowRepaid(address indexed user, bytes32 indexed strategy, uint256 borrowedRepaid, uint256 baseWithdrawn);

    event BorrowSuppliedAndDrawn(address indexed user, bytes32 indexed strategy, uint256 baseIn, uint256 debtOut);
    event UnwoundBorrow(
        address indexed user,
        bytes32 indexed strategy,
        uint256 debtRedeemed,
        uint256 debtRepaid,
        uint256 collateralRestaked
    );

    //event StrategyEmergencySet(bytes32 indexed strategy, bool on);
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

    function setOracle(IPriceOracle _oracle) external onlyRole(ROLE_GOVERNANCE) {
        oracle = _oracle;
    }

    // function setStrategy(bytes32 id, StrategyCfg calldata s) external onlyRole(ROLE_GOVERNANCE) {
    //     if (s.kind == RouteKind.DepositIdentity) {
    //         require(s.deposit.wrapper == address(0), "IDENT_NO_WRAPPER");
    //         require(s.deposit.connector != address(0), "IDENT_NEEDS_CONNECTOR");
    //         require(IExternalVaultConnector(s.deposit.connector).asset() == address(baseAsset), "CONNECTOR_NEEDS_BASE_ASSET");

    //     } else if (s.kind == RouteKind.DepositWrap1to1) {
    //         require(s.deposit.wrapper != address(0), "WRAP_NEEDS_WRAPPER");
    //         require(s.deposit.connector != address(0), "WRAP_NEEDS_CONNECTOR");
    //         require(IWrapper1to1(s.deposit.wrapper).base() == address(baseAsset), "WRAP_BASE_MISMATCH");
    //         require(
    //                 IExternalVaultConnector(s.deposit.connector).asset() == IWrapper1to1(s.deposit.wrapper).wrapped(),
    //                 "CONNECTOR_ASSET_MISMATCH"
    //             );
    //     } else if (s.kind == RouteKind.BorrowVsBase) {
    //         require(s.borrow.adapter != address(0), "BORROW_NEEDS_ADAPTER");
    //         require(s.borrow.debtAsset != address(0), "BORROW_NEEDS_DEBT_ASSET");
    //         require(IExternalVaultConnector(s.borrow.borrowedConnector).asset() == s.borrow.debtAsset, "CONNECTOR_borrowed_MISMATCH");

    //         // borrowedConnector optional if you don’t park per-user balances
    //         require(s.borrow.maxBorrowBps <= 10_000, "BPS");
    //     } else {
    //         revert("UNKNOWN_KIND");
    //     }
    //     strategies[id] = s;
    // }

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

            // borrow cfg should be inert in this kind (optional)
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

            // // borrowedConnector is OPTIONAL; if provided it must match debtAsset
            // if (s.borrow.borrowedConnector != address(0)) {
            //     require(
            //         IExternalVaultConnector(s.borrow.borrowedConnector).asset() == s.borrow.debtAsset,
            //         "CONNECTOR_borrowed_MISMATCH"
            //     );
            // }

            // safety for borrow path
            _validateBorrowSafety(s.borrow.safety);

            // deposit cfg should be inert in this kind (optional)
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

    // function setStrategyWrap(
    //     bytes32 id,
    //     address wrapper, // IWrapper1to1 or 0 (identity)
    //     address connector
    // ) external onlyRole(ROLE_GOVERNANCE) {
    //     require(connector != address(0), "CONNECTOR_ZERO");
    //     if (wrapper != address(0)) {
    //         require(IWrapper1to1(wrapper).base() == address(base), "WRAP_BASE_MISMATCH");
    //         require(
    //             IExternalVaultConnector(connector).asset() == IWrapper1to1(wrapper).wrapped(),
    //             "CONNECTOR_ASSET_MISMATCH"
    //         );
    //     } else {
    //         require(IExternalVaultConnector(connector).asset() == address(base), "CONNECTOR_NEEDS_BASE_ASSET");
    //     }
    //     strat[id] = StrategyCfg({kind: RouteKind.Wrap1to1, wrapper: wrapper, connector: connector});
    //     emit StrategySet(id, RouteKind.Wrap1to1, wrapper, connector, address(0), address(0), address(0));
    // }

    // function setStrategyBorrow(
    //     bytes32 id,
    //     address borrowVault,     // IRestrictedBorrowVault4626
    //     address borrowedConnector, // IExternalVaultConnector for the borrowed borrowed
    //     address borrowedToken
    // ) external onlyRole(ROLE_GOV) {
    //     require(borrowVault != address(0) && borrowedConnector != address(0) && borrowedToken != address(0), "ZERO");
    //     require(IExternalVaultConnector(borrowedConnector).asset() == borrowedToken, "CONNECTOR_borrowed_MISMATCH");
    //     strat[id] = StrategyCfg({
    //         kind: RouteKind.BorrowVsBase,
    //         wrapper: address(0),
    //         connector: address(0),
    //         borrowVault: borrowVault,
    //         borrowedConnector: borrowedConnector,
    //         borrowed: borrowedToken
    //     });
    //     emit StrategySet(id, RouteKind.BorrowVsBase, address(0), address(0), borrowVault, borrowedConnector, borrowedToken);
    // }

    // function setborrowedAllowed(address token, bool allowed) external onlyRole(ROLE_GOV) {
    //     borrowedAllowlist[token] = allowed; emit borrowedAllowlist(token, allowed);
    // }

    // /// @notice Called by pl right after it claims base from the SatLayer vault.
    // ///         1:1 wrap (if configured) and deposit into the external ERC-4626 connector per-user.
    // function onClaimWithStrategy(address user, uint256 baseAssets, bytes32 strategy)
    //     external
    //     nonReentrant
    //     onlyRole(ROLE_PL)
    // {
    //     require(user != address(0) && baseAssets > 0, "BAD_ARGS");
    //     StrategyCfg memory cfg = strat[strategy];
    //     require(cfg.kind == RouteKind.Wrap1to1, "NOT_WRAP");

    //     uint256 amt = baseAssets;
    //     address tokenToDeposit = address(base);
    //     if (cfg.wrapper != address(0)) {
    //         require(IWrapper1to1(cfg.wrapper).base() == address(base), "WRAP_BASE_MISMATCH");
    //         tokenToDeposit = IWrapper1to1(cfg.wrapper).wrapped();

    //         // move base to wrapper
    //         require(base.approve(cfg.wrapper, baseAssets), "APPROVE_WRAP");
    //         uint256 out = IWrapper1to1(cfg.wrapper).wrap(baseAssets);
    //         require(out == baseAssets, "WRAP_NOT_1_TO_1");
    //         amt = out;
    //     }

    //     // Deposit into external ERC-4626 connector, attributing shares to user
    //     require(IExternalVaultConnector(cfg.connector).asset() == tokenToDeposit, "CONN_ASSET_MISMATCH");
    //     require(IERC20(tokenToDeposit).approve(cfg.connector, amt), "APPROVE_CONN");
    //     IExternalVaultConnector(cfg.connector).depositFor(user, amt);

    //     emit DepositedWrap(user, strategy, baseAssets, amt);
    // }

    // /// @notice Unwind any amount (or all) for users; always price-neutral (1:1 wrap).
    // function unwindWrapAny(
    //     address user,
    //     bytes32 strategy,
    //     uint256 requestedBaseOrWrapped // pass type(uint256).max for "all"
    // ) external nonReentrant onlyKeeperOrPL {
    //     StrategyCfg memory cfg = strat[strategy];
    //     require(cfg.kind == RouteKind.Wrap1to1, "NOT_WRAP");

    //     //Determine entitlement in connector units
    //     uint256 entitlement = IExternalVaultConnector(cfg.connector).assetsOf(user);
    //     require(entitlement > 0, "NO_ENTITLEMENT");

    //     uint256 toRedeem = (requestedBaseOrWrapped == type(uint256).max)
    //         ? entitlement
    //         : (requestedBaseOrWrapped <= entitlement ? requestedBaseOrWrapped : entitlement);
    //     require(toRedeem > 0, "NOTHING_TO_REDEEM");

    //     // Redeem from connector to CG
    //     uint256 minOut = (toRedeem * (10_000 - toleranceBps)) / 10_000;

    //     (uint256 outWrappedOrBase,) = IExternalVaultConnector(cfg.connector).redeemFor(user, toRedeem, minOut);
    //     require(outWrappedOrBase >= minOut, "REDEEM_SHORTFALL");

    //     // If wrapped, unwrap back to base 1:1
    //     uint256 baseOut = outWrappedOrBase;
    //     if (cfg.wrapper != address(0)) {
    //         address w = IWrapper1to1(cfg.wrapper).wrapped();
    //         require(IERC20(w).approve(cfg.wrapper, outWrappedOrBase), "APPROVE_UNWRAP");
    //         uint256 out = IWrapper1to1(cfg.wrapper).unwrap(outWrappedOrBase);
    //         require(out >= toRedeem, "UNWRAP_SLIPPAGE");
    //         baseOut = out;
    //     }

    //     // Restake into SatLayer vault on behalf of user (reduces their pl debt)
    //     require(base.approve(pl, baseOut), "APPROVE_pl");
    //     IPL(pl).repayAndRestake(user, baseOut, strategy);

    //     emit UnwoundWrap(user, strategy, baseOut);
    // }

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

            // Wrap base → wrapped (1:1)
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

        emit DepositedWrap(user, strategy, baseIn, amt); // rename if you prefer a generic "DepositedDeposit"
    }

    /* ----------------------------------------------------------------
    * unwindDepositAny
    *  - Redeem user's position from connector
    *  - If wrapped, unwrap 1:1 back to base
    *  - Restake base to PL for the user (reduces their PL debt)
    * ---------------------------------------------------------------- */
    // function unwindDepositAny(
    //     address user,
    //     bytes32 strategy,
    //     uint256 requestedBaseOrWrapped // pass type(uint256).max for "all"
    // ) external nonReentrant onlyKeeperOrPL {
    //     StrategyCfg memory cfg = strategies[strategy];
    //     DepositSafety memory safe = depositSafety[strategy];
    //     require(
    //         cfg.kind == RouteKind.DepositIdentity || cfg.kind == RouteKind.DepositWrap1to1,
    //         "NOT_DEPOSIT_KIND"
    //     );
    //     require(cfg.deposit.connector != address(0), "CONNECTOR_ZERO");

    //     // Determine entitlement in connector units (base on Identity; wrapped on Wrap1to1)
    //     uint256 entitlement = IExternalVaultConnector(cfg.deposit.connector).assetsOf(user);
    //     require(entitlement > 0, "NO_ENTITLEMENT");

    //     uint256 toRedeem = (requestedBaseOrWrapped == type(uint256).max)
    //         ? entitlement
    //         : (requestedBaseOrWrapped <= entitlement ? requestedBaseOrWrapped : entitlement);
    //     require(toRedeem > 0, "NOTHING_TO_REDEEM");

    //     // Redeem from connector to CG (tight minOut; allow tiny tolerance if you keep one)
    //     uint256 minOut = (toRedeem * (10_000 - safe.redeemToleranceBps)) / 10_000; // redeemToleranceBps is a contract param (e.g., 0–50)
    //     (uint256 outWrappedOrBase, ) = IExternalVaultConnector(cfg.deposit.connector).redeemFor(user, toRedeem, minOut);
    //     require(outWrappedOrBase >= minOut, "REDEEM_SHORTFALL");

    //     // If wrapped path, unwrap back to base 1:1
    //     uint256 baseOut = outWrappedOrBase;
    //     if (cfg.kind == RouteKind.DepositWrap1to1) {
    //         address w = cfg.deposit.wrapper;
    //         require(w != address(0), "WRAP_NOT_SET");
    //         address wrapped = IWrapper1to1(w).wrapped();
    //         IERC20(wrapped).approve(w, outWrappedOrBase);
    //         uint256 out = IWrapper1to1(w).unwrap(outWrappedOrBase);
    //         uint16 tol = safe.emergencyMode ? safe.emergencyUnwrapBps : safe.unwrapToleranceBps;
    //         uint256 minBaseOut = (outWrappedOrBase * (10_000 - tol)) / 10_000;
    //         // strict 1:1: require(out == toRedeem) — or >= toRedeem if you want to permit fee rebates
    //         require(out >= minBaseOut, "UNWRAP_SLIPPAGE");
    //         baseOut = out;
    //     }

    //     // Restake into SatLayer vault on behalf of user (PL nets down debt and credits shares)
    //     IERC20(address(baseAsset)).approve(pl, baseOut);
    //     IPL(pl).repayAndRestake(user, baseOut, strategy);

    //     emit UnwoundWrap(user, strategy, baseOut); // rename if you prefer a generic "UnwoundDeposit"
    // }

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

        // Redeem from connector → choose tolerance based on emergency mode
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

            // // Unwrap & enforce min-out under normal/emergency settings
            // uint16 unwrapBps = safe.emergencyMode ? safe.emergencyUnwrapBps : safe.unwrapMinOutBps;
            // uint256 minBaseOut = (outWrappedOrBase * (10_000 - unwrapBps)) / 10_000;

            // unwrap guard: unwrapMinOutBps is the *minimum* fraction of wrapped you require as base
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

    //     /// @notice PL claimed base; this opens/extends a borrow position and parks borrowed borrowed per-user in a 4626 connector.
    // /// @param data encodes: (uint16 maxBorrowBps, bytes venueData)
    // function onClaimWithStrategyBorrow(
    //     address user,
    //     uint256 baseAssets,
    //     bytes32 strategy,
    //     bytes calldata data
    // ) external nonReentrant onlyRole(ROLE_PL) {
    //     require(user != address(0) && baseAssets > 0, "BAD_ARGS");
    //     StrategyCfg memory cfg = strat[strategy];
    //     require(cfg.kind == RouteKind.BorrowVsBase, "NOT_BORROW");
    //     require(borrowedAllowlist[cfg.borrowed], "borrowed_BLOCKED");

    //     (uint16 maxBorrowBps, bytes memory venueData) = abi.decode(data, (uint16, bytes));

    //     // 1) Supply base collateral & borrow borrowed (borrowed borrowed is sent to this CG)
    //     require(base.approve(cfg.borrowVault, baseAssets), "APPROVE_BORROW_VAULT");
    //     (, uint256 borrowedOut, ) = IRestrictedBorrowVault4626(cfg.borrowVault).cgSupplyAndBorrow(
    //         baseAssets, user, maxBorrowBps, venueData
    //     );

    //     if (borrowedOut > 0) {
    //         // 2) Park borrowed borrowed into per-user external 4626 connector (yield accrues)
    //         require(IExternalVaultConnector(cfg.borrowedConnector).asset() == cfg.borrowed, "borrowed_CONN_ASSET");
    //         require(IERC20(cfg.borrowed).approve(cfg.borrowedConnector, borrowedOut), "APPROVE_borrowed_CONN");
    //         IExternalVaultConnector(cfg.borrowedConnector).depositFor(user, borrowedOut);
    //     }

    //     emit BorrowOpened(user, strategy, baseAssets, maxBorrowBps, borrowedOut);
    // }

    // function onClaimWithStrategyBorrow(
    //     address user,
    //     uint256 baseIn,            // WBTC from vault
    //     bytes32 strategy,          // configured with adapter + debtAsset
    //     bytes calldata params      // abi.encode(maxBorrowBps, adapterData, connectorData)
    //     ) external onlyRole(ROLE_PL) nonReentrant {
    //     // 0) read config
    //     Config memory cfg = strat[strategy]; // {adapter, debtAsset, connector, oracle, caps...}
    //     require(cfg.adapter != address(0) && cfg.debtAsset != address(0), "CFG");

    //     // 1) supply collateral
    //     IERC20(asset).approve(cfg.adapter, baseIn);
    //     IBorrowVenueAdapter(cfg.adapter).supplyCollateral(address(asset), baseIn, /*adapterData*/ "");

    //     // 2) compute borrow amount (in debtAsset units) from oracle/caps
    //     (uint16 maxBorrowBps, bytes memory adapterData, bytes memory connectorData) =
    //         abi.decode(params,(uint16,bytes,bytes));
    //     uint256 borrowUsd  = (priceUsd(asset, baseIn) * maxBorrowBps) / 10_000;   // cap by config
    //     uint256 debtAmount = usdToToken(cfg.debtAsset, borrowUsd);                 // handle decimals

    //     // 3) borrow and route out
    //     if (debtAmount > 0) {
    //         IBorrowVenueAdapter(cfg.adapter).borrow(cfg.debtAsset, debtAmount, adapterData);
    //         if (cfg.connector != address(0)) {
    //         IERC20(cfg.debtAsset).approve(cfg.connector, debtAmount);
    //         IExternalVaultConnector(cfg.connector).depositFor(user, debtAmount);   // per-user accounting+yield
    //         } else {
    //         IERC20(cfg.debtAsset).transfer(consumer, debtAmount);                  // e.g., Nest payout
    //         }
    //     }
    //     }

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
    // function _onClaimWithStrategyBorrow(
    //     address user,
    //     uint256 baseIn,
    //     bytes32 strategy,
    //     bytes calldata params
    // ) internal  {
    //     require(user != address(0) && baseIn > 0, "BAD_ARGS");
    //     StrategyCfg memory S = strategies[strategy];
    //     require(S.kind == RouteKind.BorrowVsBase, "STRAT_KIND");

    //     BorrowCfg memory B = S.borrow;
    //     require(B.adapter != address(0) && B.debtAsset != address(0), "CFG_MISSING");

    //     (uint16 overrideBps, bytes memory adapterData, bytes memory connectorData, uint256 minBorrowOut) =
    //         abi.decode(params, (uint16, bytes, bytes, uint256));

    //     // 1) supply collateral to venue
    //     baseAsset.approve(B.adapter, baseIn);
    //     IBorrowVenueAdapter(B.adapter).supplyCollateral(address(baseAsset), baseIn, adapterData);

    //     // 2) compute borrow amount (USD-capped) and convert to debt units
    //     uint16 borrowBps = overrideBps > 0 && overrideBps < B.maxBorrowBps ? overrideBps : B.maxBorrowBps;
    //     uint256 baseUsd  = _tokenUsd(address(baseAsset), baseIn);
    //     uint256 borrowUsd= (baseUsd * borrowBps) / 10_000;
    //     uint256 debtOut  = _usdToToken(B.debtAsset, borrowUsd);
    //     require(debtOut >= minBorrowOut, "BORROW_MIN");

    //     // 3) borrow
    //     if (debtOut > 0) {
    //         IBorrowVenueAdapter(B.adapter).borrow(B.debtAsset, debtOut, adapterData);

    //         // 4) route borrowed tokens
    //         if (B.borrowedConnector != address(0)) {
    //             IERC20(B.debtAsset).approve(B.borrowedConnector, debtOut);
    //             IExternalVaultConnector(B.borrowedConnector).depositFor(user, debtOut);
    //         } else {
    //             // if you prefer direct handoff, add a consumer address & transfer here
    //             // IERC20(B.debtAsset).transfer(consumer, debtOut);
    //             revert("NO_BORROW_DEST"); // keep explicit for now
    //         }
    //     }

    //     emit BorrowSuppliedAndDrawn(user, strategy, baseIn, debtOut);
    // }
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

    // /**
    // * Emergency version of unwind:
    // * - Forces full redemption of the user’s debt entitlement (type(uint256).max)
    // * - Uses very permissive `minCollateralOut = 1`
    // * - Uses tight connector tolerance (toRedeem less tiny tolerance)
    // * - Requires either emergency latch OR live ceiling breach
    // * - Emits a distinct event and latches emergency (optional)
    // */
    // function emergencyUnwindBorrow(address user, bytes32 strategy, bytes calldata adapterData)
    //     external
    //     nonReentrant
    //     onlyKeeperOrPL
    //     returns (uint256 baseOut, uint256 repaidDebt, uint256 redeemedDebt)
    // {
    //     // Must be in emergency or actually violating ceilings
    //     require(borrowSafety[strategy].emergency || _violatesCeilings(strategy), "SAFE");

    //     // Figure out how much debt entitlement exists to set connector minOut
    //     StrategyCfg memory S = strategies[strategy];
    //     require(S.kind == RouteKind.BorrowVsBase, "NOT_BORROW");
    //     uint256 entitlement = IExternalVaultConnector(S.borrow.borrowedConnector).assetsOf(user);
    //     require(entitlement > 0, "NO_DEBT");

    //     // Set conservative mins: redeem all, accept any positive collateral,
    //     // connector minOut = entitlement minus tiny tolerance
    //     uint256 tol = (entitlement * toleranceBps) / 10_000;
    //     uint256 connectorMinOut = entitlement > tol ? (entitlement - tol) : entitlement;

    //     (baseOut, repaidDebt, redeemedDebt) =
    //         unwindBorrow(user, strategy, type(uint256).max, 1, adapterData, connectorMinOut);

    //     emit EmergencyUnwindBorrow(user, strategy, redeemedDebt, baseOut);
    // }

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
            // if you prefer direct handoff, add a consumer address & transfer here
            // IERC20(B.debtAsset).transfer(consumer, debtOut);
            revert("NO_BORROW_DEST");
        }
    }

    // Helper that checks live ceilings (best-effort; skip if adapter doesn’t expose)
    function violatesCeilings(bytes32 strat) public view returns (bool) {
        BorrowSafety memory S = borrowSafety[strat];
        if (S.maxAprBps == 0 && S.minHfBps == 0) return false;

        StrategyCfg memory C = strategies[strat];
        IRateAwareAdapter A = IRateAwareAdapter(C.borrow.adapter);

        // Try APR guard
        if (S.maxAprBps > 0) {
            try A.borrowAprBps(C.borrow.debtAsset) returns (uint16 aprBps) {
                if (aprBps > S.maxAprBps) return true;
            } catch {}
        }
        // Try HF guard
        if (S.minHfBps > 0) {
            try A.healthFactorBps() returns (uint16 hfBps) {
                if (hfBps < S.minHfBps) return true;
            } catch {}
        }
        return false;
    }

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

    // /// @notice Repay user's debt with redeemed borrowed; optionally withdraw base collateral back to SatLayer.
    // ///         No swaps here; the borrow vault computes HF and withdrawable limits.
    // /// @param requestedborrowed amount of borrowed to redeem from the borrowedConnector (type(uint256).max for all)
    // /// @param withdrawBase desired base collateral to withdraw (vault will clamp by HF)
    // function unwindBorrow(
    //     address user,
    //     bytes32 strategy,
    //     uint256 requestedborrowed,
    //     uint256 withdrawBase,
    //     bytes calldata venueData
    // ) external nonReentrant onlyKeeperOrPL {
    //     StrategyCfg memory cfg = strat[strategy];
    //     require(cfg.kind == RouteKind.BorrowVsBase, "NOT_BORROW");
    //     require(requestedborrowed > 0 || withdrawBase > 0, "NOTHING");

    //     // 1) Redeem borrowed from per-user connector (hard cap by user entitlement)
    //     uint256 entitlement = IExternalVaultConnector(cfg.borrowedConnector).assetsOf(user);
    //     uint256 toRedeem = (requestedborrowed == type(uint256).max)
    //         ? entitlement
    //         : (requestedborrowed <= entitlement ? requestedborrowed : entitlement);

    //     uint256 borrowedIn = 0;
    //     uint256 minOut = (toRedeem * (10_000 - toleranceBps)) / 10_000;
    //     if (toRedeem > 0) {
    //         (uint256 borrowedOut,) = IExternalVaultConnector(cfg.borrowedConnector).redeemFor(
    //             user, toRedeem, minOut
    //         );
    //         require(borrowedOut >= minOut, "REDEEM_SHORTFALL");
    //         borrowedIn = borrowedOut;

    //     }

    //     // 2) Repay & withdraw via borrow vault
    //     if (borrowedIn > 0) {
    //         require(IERC20(cfg.borrowed).approve(cfg.borrowVault, borrowedIn), "APPROVE_REPAY");
    //     }
    //     (uint256 repaid, uint256 baseOut) = IRestrictedBorrowVault4626(cfg.borrowVault).cgRepayAndWithdraw(
    //         user, borrowedIn, withdrawBase, venueData
    //     );

    //     // 3) Hand any withdrawn base back to PL to restake for the user
    //     if (baseOut > 0) {
    //         require(base.approve(pl, baseOut), "APPROVE_PL");
    //         IPL(pl).repayAndRestake(user, baseOut);
    //     }

    //     emit BorrowRepaid(user, strategy, repaid, baseOut);
    // }

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
     * @param minCollateralOut  Minimum base you require to proceed (slippage guard)
     * @param adapterData       Venue-specific bytes
     * @param connectorMinOut   Min debtAsset the connector must return (usually = requestedDebtIn, or tolerance-adjusted)
     */
    // function unwindBorrow(
    //     address user,
    //     bytes32 strategy,
    //     uint256 requestedDebtIn,
    //     uint256 minCollateralOut,
    //     bytes calldata adapterData,
    //     uint256 connectorMinOut
    // ) external nonReentrant onlyKeeperOrPL returns (uint256 collateralOut, uint256 repaidDebt, uint256 redeemedDebt)
    // {
    //     StrategyCfg memory S = strategies[strategy];
    //     require(S.kind == RouteKind.BorrowVsBase, "STRAT_KIND");

    //     BorrowCfg memory B = S.borrow;
    //     require(B.adapter != address(0) && B.debtAsset != address(0) && B.borrowedConnector != address(0), "CFG_MISSING");

    //     IBorrowVenueAdapter adapter_ = IBorrowVenueAdapter(B.adapter);
    //     IERC20 debt = IERC20(B.debtAsset);

    //     // 1) clip to user's entitlement and redeem from connector
    //     uint256 entitlement = IExternalVaultConnector(B.borrowedConnector).assetsOf(user);
    //     require(entitlement > 0, "NO_ENTITLEMENT");

    //     uint256 toRedeem = requestedDebtIn == type(uint256).max ? entitlement
    //                      : (requestedDebtIn <= entitlement ? requestedDebtIn : entitlement);
    //     require(toRedeem > 0, "ZERO_REQUEST");

    //     (uint256 debtIn, ) = IExternalVaultConnector(B.borrowedConnector).redeemFor(
    //         user, toRedeem, connectorMinOut // typically set = toRedeem (or small tolerance)
    //     );
    //     require(debtIn > 0, "REDEEM_FAIL");
    //     redeemedDebt = debtIn;

    //     // 2) repay venue debt
    //     uint256 beforeDebt = adapter_.debtBalance(address(debt));
    //     debt.approve(address(adapter_), debtIn);
    //     repaidDebt = adapter_.repay(address(debt), debtIn, adapterData);
    //     require(repaidDebt > 0, "REPAY_ZERO");

    //     uint256 afterDebt = adapter_.debtBalance(address(debt));
    //     // observed reduction should be >= repaid (venues may accrue interest between reads)
    //     require(beforeDebt >= afterDebt && (beforeDebt - afterDebt) >= repaidDebt, "DEBT_MISMATCH");

    //     // 3) withdraw proportional collateral (pro-rata on repaid vs pre-repay debt)
    //     uint256 collBal = adapter_.collateralBalance(address(baseAsset));
    //     uint256 proRata = beforeDebt == 0 ? 0 : (collBal * repaidDebt) / beforeDebt;

    //     // shave a tiny tolerance to avoid venue rounding reverts
    //     if (B.withdrawSlippageBps > 0) {
    //         proRata = proRata - ((proRata * B.withdrawSlippageBps) / 10_000);
    //     }
    //     require(proRata > 0, "NOTHING_TO_WITHDRAW");

    //     collateralOut = adapter_.withdrawCollateral(address(baseAsset), proRata, adapterData);
    //     require(collateralOut >= minCollateralOut, "COLLATERAL_SLIPPAGE");

    //     // 4) restake to PL on behalf of user
    //     IERC20(address(baseAsset)).approve(pl, collateralOut);
    //     IPL(pl).repayAndRestake(user, collateralOut, strategy);

    //     emit UnwoundBorrow(user, strategy, redeemedDebt, repaidDebt, collateralOut);
    // }

    // assumes the same imports, state vars, events, and structs/interfaces exist

    // assumes same imports, state, events

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

    // function _redeemFromConnector(
    //     address user,
    //     BorrowCfg storage B,
    //     uint256 requestedDebtIn,
    //     uint256 connectorMinOut
    // ) internal returns (uint256 redeemedDebt)
    // {
    //     uint256 entitlement = IExternalVaultConnector(B.borrowedConnector).assetsOf(user);
    //     require(entitlement > 0, "NO_ENTITLEMENT");

    //     uint256 toRedeem = requestedDebtIn == type(uint256).max
    //         ? entitlement
    //         : (requestedDebtIn <= entitlement ? requestedDebtIn : entitlement);
    //     require(toRedeem > 0, "ZERO_REQUEST");

    //     (uint256 debtIn, ) = IExternalVaultConnector(B.borrowedConnector).redeemFor(
    //         user, toRedeem, connectorMinOut
    //     );
    //     require(debtIn > 0, "REDEEM_FAIL");
    //     return debtIn;
    // }

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

    // function _withdrawProportionalCollateral(
    //     BorrowCfg storage B,
    //     bytes calldata adapterData,
    //     uint256 beforeDebt,
    //     uint256 repaidDebt,
    //     uint256 minCollateralOut
    // ) internal returns (uint256 collateralOut)
    // {
    //     // Recreate adapter here so we don't pass it through the call (fewer args at callsite)
    //     IBorrowVenueAdapter adapter_ = IBorrowVenueAdapter(B.adapter);

    //     uint256 collBal = adapter_.collateralBalance(address(baseAsset));
    //     uint256 proRata = beforeDebt == 0 ? 0 : (collBal * repaidDebt) / beforeDebt;

    //     if (B.withdrawSlippageBps > 0) {
    //         proRata = proRata - ((proRata * B.withdrawSlippageBps) / 10_000);
    //     }
    //     require(proRata > 0, "NOTHING_TO_WITHDRAW");

    //     collateralOut = adapter_.withdrawCollateral(address(baseAsset), proRata, adapterData);
    //     require(collateralOut >= minCollateralOut, "COLLATERAL_SLIPPAGE");
    // }

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
