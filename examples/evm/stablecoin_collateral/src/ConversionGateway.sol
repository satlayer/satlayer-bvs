// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

interface IPL {
    function repayAndRestake(address user, uint256 assets, bytes32 strategy) external;
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

/**
 * @title ConversionGateway
 * @notice Middle layer between SatLayer Vault (PositionLocker) and connectors.
 *
 * @dev
 * Purpose:
 *  - Receives SatLayer Vault’s base asset (e.g. WBTC) after async redemption.
 *  - Routes that asset into either:
 *      a) Direct deposit route (identity: base -> stable vault directly).
 *      b) Wrap route (BTC -> wBTC -> vault).
 *      c) Borrow-vs-BTC route (post-collateral borrow of stables). (future)
 *  - Manages unwind paths in reverse, ensuring stables -> base asset -> back to PositionLocker.
 *
 * Flow:
 *  1. `onClaimWithStrategy` is called by PositionLocker after claim.
 *     - CG looks up strategy config (wrapper?, connector?).
 *     - Optionally wraps base to another token (1:1).
 *     - Calls connector.depositFor(user, amount).
 *  2. On unwind:
 *     - Keeper or PositionLocker calls `unwindWrapAny` (or borrow unwind variant).
 *     - CG redeems stables from connector, unwraps if needed.
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
    IERC20 public immutable base; // SatLayer base token
    address public immutable pl; //  Position Locker
    bytes32 public constant ROLE_GOVERNANCE = keccak256("ROLE_GOVERNANCE"); // Operator
    bytes32 public constant ROLE_KEEPER = keccak256("ROLE_KEEPER");
    bytes32 public constant ROLE_PL = keccak256("ROLE_POSITION_LOCKER");
    bytes32 public constant ROLE_PAUSER = keccak256("ROLE_PAUSER");

    enum RouteKind {
        Wrap1to1
    } // TODO:BorrowVsBase

    struct StrategyCfg {
        RouteKind kind; //<-- for future other kinds
        address wrapper; // IWrapper1to1 or address(0) for identity
        address connector; // IExternalVaultConnector (holds per-user shares of wrapped/base)
    }

    mapping(bytes32 => StrategyCfg) public strat;

    event StrategySet(
        bytes32 indexed id,
        RouteKind kind,
        address wrapper,
        address connector,
        address borrowVault,
        address stableConnector,
        address stable
    );

    event DepositedWrap(address indexed user, bytes32 indexed strategy, uint256 baseIn, uint256 wrappedDeposited);
    event UnwoundWrap(address indexed user, bytes32 indexed strategy, uint256 baseOut);

    event BorrowOpened(
        address indexed user, bytes32 indexed strategy, uint256 baseIn, uint16 maxBorrowBps, uint256 stableBorrowed
    );
    event BorrowRepaid(address indexed user, bytes32 indexed strategy, uint256 stableRepaid, uint256 baseWithdrawn);

    constructor(address governance, address keeper, address pauser, address _pl, IERC20 _base) {
        require(address(_base) != address(0) && _pl != address(0), "ZERO_ADDR");
        base = _base;
        pl = _pl;
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

    function setStrategyWrap(
        bytes32 id,
        address wrapper, // IWrapper1to1 or 0 (identity)
        address connector
    ) external onlyRole(ROLE_GOVERNANCE) {
        require(connector != address(0), "CONNECTOR_ZERO");
        if (wrapper != address(0)) {
            require(IWrapper1to1(wrapper).base() == address(base), "WRAP_BASE_MISMATCH");
            require(
                IExternalVaultConnector(connector).asset() == IWrapper1to1(wrapper).wrapped(),
                "CONNECTOR_ASSET_MISMATCH"
            );
        } else {
            require(IExternalVaultConnector(connector).asset() == address(base), "CONNECTOR_NEEDS_BASE_ASSET");
        }
        strat[id] = StrategyCfg({kind: RouteKind.Wrap1to1, wrapper: wrapper, connector: connector});
        emit StrategySet(id, RouteKind.Wrap1to1, wrapper, connector, address(0), address(0), address(0));
    }

    /// @notice Called by pl right after it claims base from the SatLayer vault.
    ///         1:1 wrap (if configured) and deposit into the external ERC-4626 connector per-user.
    function onClaimWithStrategy(address user, uint256 baseAssets, bytes32 strategy)
        external
        nonReentrant
        onlyRole(ROLE_PL)
    {
        require(user != address(0) && baseAssets > 0, "BAD_ARGS");
        StrategyCfg memory cfg = strat[strategy];
        require(cfg.kind == RouteKind.Wrap1to1, "NOT_WRAP");

        uint256 amt = baseAssets;
        address tokenToDeposit = address(base);
        if (cfg.wrapper != address(0)) {
            require(IWrapper1to1(cfg.wrapper).base() == address(base), "WRAP_BASE_MISMATCH");
            tokenToDeposit = IWrapper1to1(cfg.wrapper).wrapped();

            // move base to wrapper
            require(base.approve(cfg.wrapper, baseAssets), "APPROVE_WRAP");
            uint256 out = IWrapper1to1(cfg.wrapper).wrap(baseAssets);
            require(out == baseAssets, "WRAP_NOT_1_TO_1");
            amt = out;
        }

        // Deposit into external ERC-4626 connector, attributing shares to user
        require(IExternalVaultConnector(cfg.connector).asset() == tokenToDeposit, "CONN_ASSET_MISMATCH");
        require(IERC20(tokenToDeposit).approve(cfg.connector, amt), "APPROVE_CONN");
        IExternalVaultConnector(cfg.connector).depositFor(user, amt);

        emit DepositedWrap(user, strategy, baseAssets, amt);
    }

    /// @notice Unwind any amount (or all) for users; always price-neutral (1:1 wrap).
    function unwindWrapAny(
        address user,
        bytes32 strategy,
        uint256 requestedBaseOrWrapped // pass type(uint256).max for "all"
    ) external nonReentrant onlyKeeperOrPL {
        StrategyCfg memory cfg = strat[strategy];
        require(cfg.kind == RouteKind.Wrap1to1, "NOT_WRAP");

        //Determine entitlement in connector units
        uint256 entitlement = IExternalVaultConnector(cfg.connector).assetsOf(user);
        require(entitlement > 0, "NO_ENTITLEMENT");

        uint256 toRedeem = (requestedBaseOrWrapped == type(uint256).max)
            ? entitlement
            : (requestedBaseOrWrapped <= entitlement ? requestedBaseOrWrapped : entitlement);
        require(toRedeem > 0, "NOTHING_TO_REDEEM");

        // Redeem from connector to CG
        (uint256 outWrappedOrBase,) = IExternalVaultConnector(cfg.connector).redeemFor(user, toRedeem, toRedeem);

        // If wrapped, unwrap back to base 1:1
        uint256 baseOut = outWrappedOrBase;
        if (cfg.wrapper != address(0)) {
            address w = IWrapper1to1(cfg.wrapper).wrapped();
            require(IERC20(w).approve(cfg.wrapper, outWrappedOrBase), "APPROVE_UNWRAP");
            uint256 out = IWrapper1to1(cfg.wrapper).unwrap(outWrappedOrBase);
            require(out >= toRedeem, "UNWRAP_SLIPPAGE");
            baseOut = out;
        }

        // Restake into SatLayer vault on behalf of user (reduces their pl debt)
        require(base.approve(pl, baseOut), "APPROVE_pl");
        IPL(pl).repayAndRestake(user, baseOut, strategy);

        emit UnwoundWrap(user, strategy, baseOut);
    }

    /* ========== Modifiers for callers ========== */

    modifier onlyKeeperOrPL() {
        require(hasRole(ROLE_KEEPER, msg.sender) || hasRole(ROLE_PL, msg.sender), "ONLY KEEPER OR PL");
        _;
    }
}
