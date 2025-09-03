// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {SLAYVaultV2} from "@satlayer/contracts/SLAYVaultV2.sol";

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";

interface IConversionGatewayMulti {
    function onClaimWithStrategy(address user, uint256 baseAssets, bytes32 strategy) external;
    function unwindWrapAny(address user, bytes32 strategy, uint256 requestedBaseOrWrapped, uint256 minOutAfterUnwrap)
        external;
}

/// @notice User-defined value type for strategies. Encode as bytes32 ids, e.g., keccak256("AAVE_USDC")
type StrategyId is bytes32;

/**
 * @title PositionLocker (PL)
 * @notice User-facing entry point for opting into strategies with SatLayer vault shares.
 *
 * @dev
 * Purpose:
 *  - Manages “locking” of user vault shares into specific strategies.
 *  - Tracks per-user positions:
 *      - allocatedShares: shares committed to a strategy.
 *      - transformedAssets: assets redeemed into stable strategies.
 *      - debtAssets: outstanding base assets owed to user (until unwind).
 *  - Mediates all requests/claims between Vault and ConversionGateway.
 *
 * Flow:
 *  1. Opt-in:
 *     - User deposits vault shares into PL (allocating them to a strategy).
 *     - Shares are held here, marked as `allocatedShares`.
 *  2. Request/Claim:
 *     - Keeper triggers redemption request (vault -> PL).
 *     - Claim later delivers assets directly to CG, PL books debt.
 *  3. Repay & Restake:
 *     - CG unwinds, returns base asset.
 *     - PL deposits back into Vault on behalf of user, reduces debt, increases shares.
 *  4. Opt-out:
 *     - User can withdraw unlocked shares (after debts cleared).
 *     - Strict enforcement ensures no one exits while still encumbered by strategy debt.
 *
 * Risk Controls:
 *  - Caps: per-user %, global %, epoch % (rate limit).
 *  - BufferBps: keep extra margin of shares locked against debt.
 *  - DustThreshold: ignore trivial debts to simplify exits.
 *
 * Key Points:
 *  - Users only interact here (optIn, optOut, unwind).
 */
contract PositionLocker is AccessControl, ReentrancyGuard, Pausable {
    using SafeERC20 for IERC20;

    // Immutable refs
    SLAYVaultV2 public immutable vault;
    IERC20 public immutable asset;

    bytes32 public constant ROLE_GOVERNANCE = keccak256("ROLE_GOVERNANCE"); // Operator
    bytes32 public constant ROLE_KEEPER = keccak256("ROLE_KEEPER"); // Service
    bytes32 public constant ROLE_CG = keccak256("ROLE_CONVERSION_GATEWAY");
    bytes32 public constant ROLE_PAUSER = keccak256("ROLE_PAUSER");

    // Operator sync (Operator = Governance; may also act as Keeper)
    address public operator;

    event OperatorSynced(address indexed oldOp, address indexed newOp);

    // Policy
    uint16 public perUserCapBps; // e.g., 3000 = 30%
    uint16 public globalCapBps; // e.g., 4000 = 40%
    uint16 public epochRateBps; // e.g., 500 = 5% per epoch
    uint48 public epochLength; // seconds

    // Epoch metering
    uint256 public epochStart;
    uint256 public epochOutflow; // assets requested this epoch

    // Global accounting
    uint256 public globalTransformed; // total underlying transformed across all users/strategies

    // External module
    address public conversionGateway; // CG (granted ROLE_CG on set)

    // Strategy enablement
    mapping(StrategyId => bool) public strategyEnabled;

    // Per-user & per-strategy accounting
    struct SubPos {
        uint256 allocatedShares;
        uint256 transformedAssets;
        uint256 debtAssets;
    }

    struct Position {
        uint256 totalShares;
        uint256 transformedTotal;
        mapping(StrategyId => SubPos) sub;
    }

    mapping(address => Position) internal positions;

    // Requests
    struct RequestInfo {
        address owner;
        StrategyId strategy;
        uint256 shares;
        uint256 createdAt;
    }

    mapping(uint256 => RequestInfo) public requests;

    // Dust/Buffer for partial unlocks
    uint256 public dustThreshold; // treat <= dust as 0
    uint16 public bufferBps = 50; // keep +0.50% shares to cover drift

    // Events
    event CapsUpdated(uint16 perUser, uint16 globalCap, uint16 epochRate, uint48 epochLen);
    event Paused(bool on);
    event ConversionGatewayUpdated(address indexed cg);
    event StrategyEnabled(StrategyId indexed strat, bool enabled);

    event OptIn(address indexed user, uint256 shares, StrategyId indexed strategy);
    event Requested(address indexed user, uint256 indexed reqId, StrategyId indexed strategy, uint256 shares);
    event Claimed(uint256 indexed reqId, address indexed user, StrategyId indexed strategy, uint256 assetsOut);
    event ReDeposited(address indexed user, StrategyId indexed strategy, uint256 assetsIn, uint256 sharesOut);
    event OptOut(address indexed user, uint256 shares, StrategyId indexed strategy);

    mapping(bytes32 => uint256) public strategyTransformed; // total debt across users per strategy
    uint16 public userUnwindHeadroomBps = 100; // 1%

    constructor(SLAYVaultV2 _vault) {
        require(address(_vault) != address(0), "VAULT_ZERO");
        vault = _vault;
        asset = IERC20(_vault.asset());

        // defaults (tune via gov)
        perUserCapBps = 3000;
        globalCapBps = 4000;
        epochRateBps = 500;
        epochLength = 1 days;
        epochStart = block.timestamp;

        _syncOperatorInternal(); // grants roles from delegated()
    }

    function syncOperator() external {
        _syncOperatorInternal();
    }

    function _syncOperatorInternal() internal {
        address newOp = vault.delegated();
        require(newOp != address(0), "NO_OPERATOR");
        address old = operator;
        if (newOp == old) return;

        if (old != address(0)) {
            revokeRole(ROLE_GOVERNANCE, old);
            revokeRole(ROLE_KEEPER, old);
            revokeRole(ROLE_PAUSER, old);
        }
        operator = newOp;
        _grantRole(ROLE_GOVERNANCE, newOp);
        _grantRole(ROLE_KEEPER, newOp); // allow service==operator to run keeper flows
        _grantRole(ROLE_PAUSER, newOp);
        _pause();
        emit OperatorSynced(old, newOp);
    }

    /* ===== Governance ===== */
    function setUserUnwindHeadroomBps(uint16 bps) external onlyRole(ROLE_GOVERNANCE) {
        require(bps <= 1000, "HEADROOM_BIG"); // ≤10%
        userUnwindHeadroomBps = bps;
    }

    function setCaps(uint16 _perUser, uint16 _global, uint16 _epochRate, uint48 _epochLen)
        external
        onlyRole(ROLE_GOVERNANCE)
    {
        require(_perUser <= 10_000 && _global <= 10_000 && _epochRate <= 10_000, "BPS_RANGE");
        perUserCapBps = _perUser;
        globalCapBps = _global;
        epochRateBps = _epochRate;
        epochLength = _epochLen;
        emit CapsUpdated(_perUser, _global, _epochRate, _epochLen);
    }

    function setConversionGateway(address _cg) external onlyRole(ROLE_GOVERNANCE) {
        require(_cg != address(0), "CG_ZERO");
        conversionGateway = _cg;
        _grantRole(ROLE_CG, _cg);
        emit ConversionGatewayUpdated(_cg);
    }

    function setStrategyEnabled(StrategyId s, bool enabled) external onlyRole(ROLE_GOVERNANCE) {
        strategyEnabled[s] = enabled;
        emit StrategyEnabled(s, enabled);
    }

    function setDustAndBuffer(uint256 _dust, uint16 _bufferBps) external onlyRole(ROLE_GOVERNANCE) {
        require(_bufferBps <= 1000, "BUFFER_TOO_BIG"); // ≤10%
        dustThreshold = _dust;
        bufferBps = _bufferBps;
    }

    function setPaused(bool on) external onlyRole(ROLE_PAUSER) {
        if (on) _pause();
        else _unpause();
    }

    function grantKeeper(address k) external onlyRole(ROLE_GOVERNANCE) {
        _grantRole(ROLE_KEEPER, k);
    }

    function setPauser(address p) external onlyRole(ROLE_GOVERNANCE) {
        _grantRole(ROLE_PAUSER, p);
    }

    /* ===== User: Opt-In / Opt-Out ===== */

    /// @notice User opts into a specific strategy by locking vault shares inside the PL.
    function optIn(uint256 shares, StrategyId strategy) external nonReentrant whenNotPaused {
        require(shares > 0, "ZERO");
        require(strategyEnabled[strategy], "STRAT_DISABLED");

        // Pull vault shares into PL
        IERC20(address(vault)).safeTransferFrom(msg.sender, address(this), shares);

        // Book under user+strategy
        Position storage P = positions[msg.sender];
        P.totalShares += shares;
        P.sub[strategy].allocatedShares += shares;

        emit OptIn(msg.sender, shares, strategy);
    }

    /// @notice Strict: unlock ALL shares only if total user debt is zero across strategies.
    function optOutAll(StrategyId[] calldata strategies) external nonReentrant {
        Position storage P = positions[msg.sender];

        uint256 totalDebt;
        for (uint256 i = 0; i < strategies.length; ++i) {
            uint256 d = P.sub[strategies[i]].debtAssets;
            if (d > dustThreshold) totalDebt += d;
        }
        // Require user have no debt
        require(totalDebt == 0, "OUTSTANDING_DEBT");

        uint256 amount = P.totalShares;
        require(amount > 0, "NO_SHARES");

        // Zero allocations per strategy
        for (uint256 i = 0; i < strategies.length; ++i) {
            P.sub[strategies[i]].allocatedShares = 0;
        }
        P.totalShares = 0;

        SafeERC20.safeTransfer(IERC20(address(vault)), msg.sender, amount);
        emit OptOut(msg.sender, amount, StrategyId.wrap(bytes32(0)));
    }

    /// @notice Partial: unlock some unencumbered shares from a specific strategy.
    function optOutFromStrategy(uint256 shares, StrategyId strategy) external nonReentrant {
        uint256 maxOut = unlockable(msg.sender, strategy);
        require(shares > 0 && shares <= maxOut, "LOCKED_BY_DEBT");

        Position storage P = positions[msg.sender];
        SubPos storage S = P.sub[strategy];

        S.allocatedShares -= shares;
        P.totalShares -= shares;

        SafeERC20.safeTransfer(IERC20(address(vault)), msg.sender, shares);
        emit OptOut(msg.sender, shares, strategy);
    }

    /// @notice User can call when they want to unwind their assets.
    function userUnwindWrapAny(
        StrategyId strategy,
        uint256 requestedBaseOrWrapped, // pass type(uint256).max for "all"
        uint256 minOutAfterUnwrap // usually == requested if 1:1
    ) external nonReentrant whenNotPaused {
        require(strategyEnabled[strategy], "STRAT_DISABLED");
        IConversionGatewayMulti(conversionGateway).unwindWrapAny(
            msg.sender, StrategyId.unwrap(strategy), requestedBaseOrWrapped, minOutAfterUnwrap
        );
    }

    /* ===== Keeper: Request / Claim / Repay ===== */

    /// @notice Keeper asks to redeem `shares` from `user` under `strategy`.
    function requestFor(address user, uint256 shares, StrategyId strategy)
        external
        onlyRole(ROLE_KEEPER)
        nonReentrant
        whenNotPaused
        returns (uint256 reqId)
    {
        require(strategyEnabled[strategy], "STRAT_DISABLED");
        require(conversionGateway != address(0), "CG_NOT_SET");

        Position storage P = positions[user];
        SubPos storage S = P.sub[strategy];

        require(shares > 0 && shares <= S.allocatedShares, "SHARES_OOB");

        _rollEpoch();

        // Cap checks
        uint256 totalAssets_ = vault.totalAssets();
        uint256 assetsReq = vault.convertToAssets(shares);

        // Per-user cap uses total transformed across strategies + this request
        uint256 userCap = (vault.convertToAssets(P.totalShares) * perUserCapBps) / 10_000;
        require(P.transformedTotal + assetsReq <= userCap, "PER_USER_CAP");

        uint256 globalCap = (totalAssets_ * globalCapBps) / 10_000;
        require(globalTransformed + assetsReq <= globalCap, "GLOBAL_CAP");

        uint256 epochLimit = (totalAssets_ * epochRateBps) / 10_000;
        require(epochOutflow + assetsReq <= epochLimit, "EPOCH_CAP");

        // Move shares out
        S.allocatedShares -= shares;
        P.totalShares -= shares;

        // Open async redemption (controller = PL, owner = PL)
        SafeERC20.safeIncreaseAllowance(IERC20(address(vault)), address(vault), shares);
        reqId = vault.requestRedeem(shares, address(this), address(this));

        requests[reqId] = RequestInfo({owner: user, strategy: strategy, shares: shares, createdAt: block.timestamp});

        epochOutflow += assetsReq;
        emit Requested(user, reqId, strategy, shares);
    }

    /// @notice Keeper claims a ready request; vault pays underlying directly to CG; PL books debt and notifies CG.
    /// @param reqId Redemption request id (opened via requestFor)
    function claimTo(uint256 reqId)
        external
        onlyRole(ROLE_KEEPER)
        nonReentrant
        whenNotPaused
        returns (uint256 assetsOut)
    {
        RequestInfo memory r = requests[reqId];
        require(r.owner != address(0), "REQ_UNKNOWN");

        // Vault transfers UNDERLYING to CG
        assetsOut = vault.redeem(r.shares, conversionGateway, address(this));
        delete requests[reqId];

        // Book user debt/transformed (per strategy + aggregate)
        Position storage P = positions[r.owner];
        SubPos storage S = P.sub[r.strategy];

        S.debtAssets += assetsOut;
        S.transformedAssets += assetsOut;
        P.transformedTotal += assetsOut;
        globalTransformed += assetsOut;
        strategyTransformed[StrategyId.unwrap(r.strategy)] += assetsOut;

        emit Claimed(reqId, r.owner, r.strategy, assetsOut);

        IConversionGatewayMulti(conversionGateway).onClaimWithStrategy(
            r.owner, assetsOut, StrategyId.unwrap(r.strategy)
        );
    }

    /// @notice CG returns vault asset back for user/strategy; PL re-deposits and nets down debts.
    function repayAndRestake(address user, uint256 assets, StrategyId strategy)
        external
        onlyRole(ROLE_CG)
        nonReentrant
        returns (uint256 sharesOut)
    {
        require(assets > 0, "ZERO");

        // Pull underlying from CG
        asset.safeTransferFrom(msg.sender, address(this), assets);

        // Deposit back into the vault
        asset.safeIncreaseAllowance(address(vault), assets);
        sharesOut = vault.deposit(assets, address(this));

        // Accounting
        Position storage P = positions[user];
        SubPos storage S = P.sub[strategy];

        // Reduce debts
        S.debtAssets = S.debtAssets > assets ? S.debtAssets - assets : 0;
        S.transformedAssets = S.transformedAssets > assets ? S.transformedAssets - assets : 0;

        // Aggregate transformed
        P.transformedTotal = P.transformedTotal > assets ? P.transformedTotal - assets : 0;
        globalTransformed = globalTransformed > assets ? globalTransformed - assets : 0;

        // Increase share holdings
        P.totalShares += sharesOut;
        S.allocatedShares += sharesOut;
        strategyTransformed[StrategyId.unwrap(strategy)] = strategyTransformed[StrategyId.unwrap(strategy)] > assets
            ? strategyTransformed[StrategyId.unwrap(strategy)] - assets
            : 0;

        emit ReDeposited(user, strategy, assets, sharesOut);
    }

    /* ===== Views & Helpers ===== */

    /// @notice Shares currently unlockable from a specific strategy (keeps enough to cover debt + buffer).
    function unlockable(address user, StrategyId s) public view returns (uint256) {
        Position storage P = positions[user];
        SubPos storage S = P.sub[s];

        uint256 debt = S.debtAssets;
        if (debt <= dustThreshold) debt = 0;
        if (debt == 0) return S.allocatedShares;

        //Cover in case rounding/fee
        uint256 encShares = vault.convertToShares(debt);
        encShares = encShares + (encShares * bufferBps) / 10_000;
        return S.allocatedShares > encShares ? (S.allocatedShares - encShares) : 0;
    }

    function userTotals(address user)
        external
        view
        returns (uint256 totalShares, uint256 transformedTotal, uint256 epochStart_, uint256 epochOutflow_)
    {
        Position storage P = positions[user];
        return (P.totalShares, P.transformedTotal, epochStart, epochOutflow);
    }

    function _rollEpoch() internal {
        if (block.timestamp >= epochStart + epochLength) {
            epochStart = block.timestamp;
            epochOutflow = 0;
        }
    }

    function userDebtForStrategy(address u, bytes32 s) external view returns (uint256) {
        return positions[u].sub[StrategyId.wrap(s)].debtAssets;
    }

    function strategyTransformedTotal(bytes32 s) external view returns (uint256) {
        return strategyTransformed[s];
    }
}
