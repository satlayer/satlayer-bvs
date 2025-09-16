// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {ISLAYRegistryV2} from "@satlayer/contracts/src/interface/ISLAYRegistryV2.sol";
import {RelationshipV2} from "@satlayer/contracts/src/RelationshipV2.sol";

/**
 * @title StablecoinCollateralBVS (Selector-Based BVS)
 * @notice Governance opens requests that specify (target, selector, arg policy).
 *         Operators attest with a tx hash; service verifies off-chain, then finalizes.
 */
contract StablecoinCollateralBVS is Ownable {
    ISLAYRegistryV2 public immutable slayRegistry;

    enum ReqStatus {
        Open,
        Finalized,
        Expired,
        Cancelled
    }
    enum MatchMode {
        NONE,
        EXACT,
        PREFIX
    }
    enum CompletionMode {
        ANY,
        ALL,
        AT_LEAST_K
    }

    struct Action {
        address target;
        bytes4 selector;
        bytes expectedArgs; // bounded
        bytes32 expectedArgsHash; // keccak256(expectedArgs)
        bytes extraData; // bounded, optional
        MatchMode matchMode;
    }

    struct Request {
        uint64 chainId;
        CompletionMode completion;
        uint16 kRequired; // only if completion == AT_LEAST_K
        uint16 quorumBps; // per-action quorum
        uint16 minCount; // floor per action
        uint48 createdAt;
        uint48 expiresAt;
        ReqStatus status;
        uint16 attestedCount; // optional aggregate counter (not used in finalize)
        uint32 finalizedAt;
        bool hasOperatorAllowlist; // if true, must be in operatorAllowed
    }

    // reqId => Request / Actions
    mapping(uint256 => Request) public requests;
    mapping(uint256 => Action[]) public requestActions;

    // reqId => operator => allowed (if allowlist enabled)
    mapping(uint256 => mapping(address => bool)) public operatorAllowed;

    // reqId => actionIdx => attest count
    mapping(uint256 => mapping(uint256 => uint16)) public actionAttestCount;

    // reqId => actionIdx => operator => seen?
    mapping(uint256 => mapping(uint256 => mapping(address => bool))) public hasAttested;

    // reqId => actionIdx => operator => txHash
    mapping(uint256 => mapping(uint256 => mapping(address => bytes32))) public attestedTxHash;

    uint256 public constant MAX_EXPECTED_ARGS = 256;
    uint256 public constant MAX_EXTRA = 256;

    uint256 public nextRequestId;
    uint256 public activeOperatorCount;

    /* --------------------------------- Events -------------------------------- */

    event RequestOpened(
        uint256 indexed id,
        uint64 chainId,
        CompletionMode completion,
        uint16 kRequired,
        uint16 quorumBps,
        uint16 minCount,
        uint48 expiresAt
    );

    event RequestActionAdded(
        uint256 indexed id,
        uint256 indexed index,
        address target,
        bytes4 selector,
        MatchMode matchMode,
        bytes32 expectedArgsHash
    );

    event RequestOperatorsAllowlisted(uint256 indexed id, uint256 count);

    event Attested(
        uint256 indexed requestId,
        uint256 indexed actionIndex,
        address indexed operator,
        bytes32 txHash,
        bytes extraData
    );

    event RequestExpired(uint256 indexed id);
    event RequestFinalized(uint256 indexed requestId, uint16 requiredCount, uint256 actions, uint32 finalizedAt);
    event RequestCanceled(uint256 indexed requestId, string reason);

    event Unsolicited(
        uint64 indexed chainId,
        address indexed operator,
        address indexed target,
        bytes4 selector,
        bytes32 txHash,
        bytes args
    );

    event OperatorRegistered(address indexed operator, bool added);

    constructor(ISLAYRegistryV2 registry, address owner_) Ownable(owner_) {
        require(address(registry) != address(0), "registry=0");
        slayRegistry = registry;
        registry.registerAsService("https://service.satlayer.example", "Stablecoin Collateral Service");
    }

    /* ----------------------------- Operator admin ---------------------------- */

    function registerOperator(address op) external onlyOwner {
        slayRegistry.registerOperatorToService(op);
        emit OperatorRegistered(op, true);
    }

    function deregisterOperator(address op) external onlyOwner {
        slayRegistry.deregisterOperatorFromService(op);
        emit OperatorRegistered(op, false);
    }

    function isActiveOperatorAt(address op, uint256 ts) public view returns (bool) {
        return slayRegistry.getRelationshipStatusAt(address(this), op, uint32(ts)) == RelationshipV2.Status.Active;
    }

    /* ------------------------------- Open request ---------------------------- */

    /**
     * @param chainId     Chain where the executions must occur.
     * @param actions     Array of actions (target/selector/expectedArgs/matchMode/extraData). expectedArgs <= MAX_EXPECTED_ARGS.
     * @param completion  ANY / ALL / AT_LEAST_K.
     * @param kRequired   When completion == AT_LEAST_K, require at least K actions to be satisfied.
     * @param quorumBps   0..10000. If >0 compute ceil(activeOps * bps/10000) per-action.
     * @param minCount    Absolute floor per-action. Must be >0 if quorumBps == 0.
     * @param ttlSeconds  0 = no expiry; otherwise now + ttl.
     * @param allowedOps  Optional operator allowlist; empty => any active operator.
     */
    function openRequest(
        uint64 chainId,
        Action[] calldata actions,
        CompletionMode completion,
        uint16 kRequired,
        uint16 quorumBps,
        uint16 minCount,
        uint48 ttlSeconds,
        address[] calldata allowedOps
    ) external onlyOwner returns (uint256 id) {
        require(actions.length > 0, "no actions");
        require(quorumBps <= 10_000, "bps>100%");
        require(quorumBps > 0 || minCount > 0, "no quorum");
        if (completion == CompletionMode.AT_LEAST_K) {
            require(kRequired > 0 && kRequired <= actions.length, "bad K");
        } else {
            require(kRequired == 0, "K only for AT_LEAST_K");
        }

        id = nextRequestId++;
        Request storage R = requests[id];
        R.chainId = chainId;
        R.completion = completion;
        R.kRequired = kRequired;
        R.quorumBps = quorumBps;
        R.minCount = minCount;
        R.createdAt = uint48(block.timestamp);
        R.expiresAt = ttlSeconds == 0 ? 0 : uint48(block.timestamp) + ttlSeconds;
        R.status = ReqStatus.Open;

        Action[] storage dst = requestActions[id];
        for (uint256 i = 0; i < actions.length; i++) {
            Action calldata A = actions[i];
            require(A.target != address(0), "target=0");
            require(A.extraData.length <= MAX_EXTRA, "extra too large");

            Action memory B;
            B.target = A.target;
            B.selector = A.selector;
            B.matchMode = A.matchMode;
            B.expectedArgs = A.expectedArgs;
            B.expectedArgsHash = keccak256(A.expectedArgs);
            B.extraData = A.extraData;

            dst.push(B);
            emit RequestActionAdded(id, i, B.target, B.selector, B.matchMode, B.expectedArgsHash);
        }

        if (allowedOps.length > 0) {
            R.hasOperatorAllowlist = true;
            for (uint256 j = 0; j < allowedOps.length; j++) {
                address op = allowedOps[j];
                require(op != address(0), "op=0");
                operatorAllowed[id][op] = true;
            }
            emit RequestOperatorsAllowlisted(id, allowedOps.length);
        }

        emit RequestOpened(id, chainId, completion, kRequired, quorumBps, minCount, R.expiresAt);
    }

    /* ----------------------------- Threshold logic --------------------------- */

    function _requiredCount(uint256 requestId) internal view returns (uint16 required) {
        Request storage R = requests[requestId];

        // Prefer a count-at-time API to freeze quorum at creation.
        uint256 active;

        active = slayRegistry.getActiveOperatorCount(address(this));

        uint256 reqByQuorum = R.quorumBps > 0
            ? (active * uint256(R.quorumBps) + 9_999) / 10_000 // ceil
            : 0;

        uint256 req = reqByQuorum;
        if (R.minCount > req) req = R.minCount;

        require(req <= type(uint16).max, "threshold overflow");
        required = uint16(req);
        require(required > 0, "no threshold");
    }

    /* --------------------------------- Attest -------------------------------- */

    /**
     * @notice Operator attests that they executed an action. Service verifies off-chain.
     */
    function attest(uint256 requestId, uint256 actionIndex, bytes32 txHash, bytes calldata extraData) external {
        require(txHash != bytes32(0), "txHash=0");
        require(extraData.length <= MAX_EXTRA, "extra too big");

        Request storage R = requests[requestId];
        require(R.status == ReqStatus.Open, "req not open");

        if (R.expiresAt != 0 && block.timestamp > R.expiresAt) {
            R.status = ReqStatus.Expired;
            emit RequestExpired(requestId);
        } else {
            Action[] storage actions = requestActions[requestId];
            require(actionIndex < actions.length, "bad actionIndex");

            if (R.hasOperatorAllowlist) {
                require(operatorAllowed[requestId][msg.sender], "op not allowlisted");
            }

            require(isActiveOperatorAt(msg.sender, R.createdAt), "operator not active@createdAt");
            require(!hasAttested[requestId][actionIndex][msg.sender], "dup attestation");

            hasAttested[requestId][actionIndex][msg.sender] = true;
            attestedTxHash[requestId][actionIndex][msg.sender] = txHash;

            actionAttestCount[requestId][actionIndex] += 1;
            unchecked {
                R.attestedCount += 1;
            }

            emit Attested(requestId, actionIndex, msg.sender, txHash, extraData);
        }
    }

    /* --------------------------- Finalize / Cancel --------------------------- */

    function finalizeRequest(uint256 requestId) external onlyOwner {
        Request storage R = requests[requestId];
        require(R.status == ReqStatus.Open, "not open");

        uint16 required = _requiredCount(requestId);
        Action[] storage acts = requestActions[requestId];
        uint256 n = acts.length;

        // Tally how many actions have met the per-action quorum/threshold
        uint256 satisfied = 0;
        for (uint256 i = 0; i < n; i++) {
            if (actionAttestCount[requestId][i] >= required) {
                unchecked {
                    satisfied++;
                }
            }
        }

        // Honor the completion policy
        if (R.completion == CompletionMode.ALL) {
            require(satisfied == n, "ALL:not all actions satisfied");
        } else if (R.completion == CompletionMode.ANY) {
            require(satisfied >= 1, "ANY:no action satisfied");
        } else {
            // AT_LEAST_K
            require(R.kRequired > 0 && R.kRequired <= n, "K:bad config");
            require(satisfied >= R.kRequired, "K:insufficient satisfied");
        }

        R.status = ReqStatus.Finalized;
        R.finalizedAt = uint32(block.timestamp);

        emit RequestFinalized(requestId, required, n, R.finalizedAt);
    }

    function cancelRequest(uint256 requestId, string calldata reason) external onlyOwner {
        Request storage R = requests[requestId];
        require(R.status == ReqStatus.Open, "not open");
        R.status = ReqStatus.Cancelled;
        emit RequestCanceled(requestId, reason);
    }

    /* ------------------------------ Unsolicited ------------------------------ */

    /// Operators can self-report actions even without an open request (light index).
    function notifyUnsolicited(uint64 chainId, address target, bytes4 selector, bytes32 txHash, bytes calldata args)
        external
    {
        require(
            slayRegistry.getRelationshipStatus(address(this), msg.sender) == RelationshipV2.Status.Active,
            "operator not active"
        );
        require(txHash != bytes32(0), "txHash=0");
        emit Unsolicited(chainId, msg.sender, target, selector, txHash, args);
    }

    /* --------------------------------- Views --------------------------------- */
    function canFinalize(uint256 requestId)
        external
        view
        returns (bool ok, uint16 required, uint16[] memory counts, uint256 satisfied)
    {
        Request storage R = requests[requestId];
        if (R.status != ReqStatus.Open) return (false, 0, counts, 0);

        Action[] storage acts = requestActions[requestId];
        counts = new uint16[](acts.length);
        required = _requiredCount(requestId);

        for (uint256 i = 0; i < acts.length; i++) {
            uint16 c = actionAttestCount[requestId][i];
            counts[i] = c;
            if (c >= required) satisfied++;
        }

        if (R.completion == CompletionMode.ALL) {
            ok = (satisfied == acts.length);
        } else if (R.completion == CompletionMode.ANY) {
            ok = (satisfied >= 1);
        } else {
            ok = (R.kRequired > 0 && satisfied >= R.kRequired);
        }
    }

    function checkRequestStatus(uint256 requestId) public view returns (ReqStatus status) {
        Request storage R = requests[requestId];
        return R.status;
    }

    function actionCount(uint256 requestId) external view returns (uint256) {
        return requestActions[requestId].length;
    }

    function getAction(uint256 requestId, uint256 index)
        external
        view
        returns (
            address target,
            bytes4 selector,
            bytes memory expectedArgs,
            bytes32 expectedArgsHash,
            bytes memory extraData,
            MatchMode matchMode
        )
    {
        Action storage A = requestActions[requestId][index];
        return (A.target, A.selector, A.expectedArgs, A.expectedArgsHash, A.extraData, A.matchMode);
    }

    function getRequest(uint256 requestId) external view returns (Request memory) {
        return requests[requestId];
    }
}
