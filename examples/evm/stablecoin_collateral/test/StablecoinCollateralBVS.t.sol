// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";

import {StablecoinCollateralBVS} from "../src/StablecoinCollateralBVS.sol";
import {ISLAYRegistryV2} from "@satlayer/contracts/src/interface/ISLAYRegistryV2.sol";
import {RelationshipV2} from "@satlayer/contracts/src/RelationshipV2.sol";


import {TestSuiteV2} from "@satlayer/contracts/test/TestSuiteV2.sol";


contract StablecoinCollateralBVSTest is Test, TestSuiteV2 {
    StablecoinCollateralBVS svc;

    address public op1      = makeAddr("op1");
    address public op2      = makeAddr("op2");
    address public op3      = makeAddr("op3");
    address public target1  = makeAddr("target1");
    address public target2  = makeAddr("target2");
    address public target3  = makeAddr("target3");

    function setUp() public override {
        TestSuiteV2.setUp(); // gives us: registry, router, owner (router owner), etc.

        // Deploy service with SatLayer registry reference and our chosen owner
        svc = new StablecoinCollateralBVS(registry, owner);

        // Register three operators under this Service (must be onlyOwner)

        // register operator
        vm.prank(op1);
        registry.registerAsOperator("https://example.com", "Operator Y");
        vm.prank(op2);
        registry.registerAsOperator("https://example.com", "Operator Z");
        vm.prank(op3);
        registry.registerAsOperator("https://example.com", "Operator 3");
        vm.prank(op1);
        registry.registerServiceToOperator(address(svc));
        vm.prank(op2);
        registry.registerServiceToOperator(address(svc));
        vm.prank(op3);
        registry.registerServiceToOperator(address(svc));

        vm.startPrank(owner);
        svc.registerOperator(op1);
        svc.registerOperator(op2);
        svc.registerOperator(op3);

        vm.stopPrank();

    }

    /* ------------------------- helpers ------------------------- */

    function _action(
        address t,
        bytes4 sel,
        bytes memory args,
        StablecoinCollateralBVS.MatchMode mode
    ) internal pure returns (StablecoinCollateralBVS.Action memory A) {
        A.target = t;
        A.selector = sel;
        A.expectedArgs = args;          // service will bound-check & hash
        A.expectedArgsHash = bytes32(0);
        A.matchMode = mode;
        A.extraData = "";
    }

    function _open(
        StablecoinCollateralBVS.CompletionMode completion,
        uint16 kRequired,
        uint16 quorumBps,
        uint16 minCount,
        uint48 ttlSeconds,
        StablecoinCollateralBVS.Action[] memory actions,
        address[] memory allowOps
    ) internal returns (uint256 id) {
        vm.prank(owner);
        id = svc.openRequest(
            uint64(block.chainid),
            actions,
            completion,
            kRequired,
            quorumBps,
            minCount,
            ttlSeconds,
            allowOps
        );
    }

    function _attest(address op, uint256 id, uint256 actionIdx, bytes32 txh) internal {
        vm.prank(op);
        svc.attest(id, actionIdx, txh, "");
    }

    /* --------------------------- tests --------------------------- */

    function test_ANY_finalizes_when_one_action_meets_threshold() public {
        StablecoinCollateralBVS.Action[] memory actions =
            new StablecoinCollateralBVS.Action [] (2);

        actions[0] = _action(target1, bytes4(0xAAAA0001), hex"1122", StablecoinCollateralBVS.MatchMode.EXACT);
        actions[1] = _action(target2, bytes4(0xBBBB0002), hex"3344", StablecoinCollateralBVS.MatchMode.EXACT);

        // quorumBps=0, minCount=2 → per-action required = 2
        address [] memory empty = new address[](0);
        uint256 id = _open(
            StablecoinCollateralBVS.CompletionMode.ANY,
            0,
            0,
            2,
            1 days,
            actions,
            empty 
        );

        // Only satisfy action[0]
        _attest(op1, id, 0, bytes32(uint256(0x1)));
        _attest(op2, id, 0, bytes32(uint256(0x2)));

        (bool ok, , , uint256 satisfied) = svc.canFinalize(id);
        assertEq(satisfied, 1, "exactly one action satisfied");

        // ANY → should finalize
        vm.prank(owner);
        svc.finalizeRequest(id);

        // After finalization, canFinalize should report status != Open
        (ok, , ,  satisfied) = svc.canFinalize(id);
        assertTrue(!ok, "not Open after finalize");
        
    }

    function test_ALL_requires_all_actions_meet_threshold() public {
        StablecoinCollateralBVS.Action[] memory actions =
            new StablecoinCollateralBVS.Action [] (2);
        actions[0] = _action(target1, bytes4(0xCAFE0001), "", StablecoinCollateralBVS.MatchMode.NONE);
        actions[1] = _action(target2, bytes4(0xCAFE0002), "", StablecoinCollateralBVS.MatchMode.NONE);

        address [] memory empty = new address[](0);
        uint256 id = _open(
            StablecoinCollateralBVS.CompletionMode.ALL,
            0,
            0,
            2,  // need 2 attests per action
            1 days,
            actions,
            empty
        );

        // Satisfy only action[0]
        _attest(op1, id, 0, bytes32(uint256(1)));
        _attest(op2, id, 0, bytes32(uint256(2)));

        vm.prank(owner);
        vm.expectRevert(bytes("ALL:not all actions satisfied"));
        svc.finalizeRequest(id);

        // Now satisfy action[1] too
        _attest(op1, id, 1, bytes32(uint256(3)));
        _attest(op3, id, 1, bytes32(uint256(4)));

        vm.prank(owner);
        svc.finalizeRequest(id);
    }

    function test_AT_LEAST_K_requires_k_actions_meet_threshold() public {
        StablecoinCollateralBVS.Action[] memory actions =
            new StablecoinCollateralBVS.Action [] (3);
        actions[0] = _action(target1, bytes4(0xAAAA0001), "", StablecoinCollateralBVS.MatchMode.NONE);
        actions[1] = _action(target2, bytes4(0xAAAA0002), "", StablecoinCollateralBVS.MatchMode.NONE);
        actions[2] = _action(target3, bytes4(0xAAAA0003), "", StablecoinCollateralBVS.MatchMode.NONE);

        address [] memory empty = new address[](0);

        // k=2; per-action required = 2
        uint256 id = _open(
            StablecoinCollateralBVS.CompletionMode.AT_LEAST_K,
            2,
            0,
            2,
            1 days,
            actions,
           empty
        );

        // Satisfy 2 actions
        _attest(op1, id, 0, bytes32(uint256(1)));
        _attest(op2, id, 0, bytes32(uint256(2)));

        _attest(op2, id, 1, bytes32(uint256(3)));
        _attest(op3, id, 1, bytes32(uint256(4)));

        vm.prank(owner);
        svc.finalizeRequest(id);
    }

    function test_allowlist_enforced() public {
        StablecoinCollateralBVS.Action[] memory actions =
            new StablecoinCollateralBVS.Action [] (1);
        actions[0] = _action(target1, bytes4(0xDEAD0001), "", StablecoinCollateralBVS.MatchMode.NONE);

        // address;
        // allow[0] = op1;

        address [] memory allow = new address[](1);
        allow[0] = op1;

        uint256 id = _open(
            StablecoinCollateralBVS.CompletionMode.ANY,
            0, 0, 1, 1 days,
            actions,
            allow
        );

        vm.prank(op2);
        vm.expectRevert(bytes("op not allowlisted"));
        svc.attest(id, 0, bytes32(uint256(1)), "");

        // Allowlisted op1 can attest & finalize (ANY)
        _attest(op1, id, 0, bytes32(uint256(2)));
        vm.prank(owner);
        svc.finalizeRequest(id);
    }

    function test_duplicate_attest_rejected() public {
        StablecoinCollateralBVS.Action[] memory actions =new StablecoinCollateralBVS.Action [](1);

        

        actions[0] = _action(target1, bytes4(0xAABBCCDD), "", StablecoinCollateralBVS.MatchMode.NONE);

        address [] memory allow = new address[](0);
        uint256 id = _open(
            StablecoinCollateralBVS.CompletionMode.ANY,
            0, 0, 1, 1 days,
            actions,
           allow
        );

        _attest(op1, id, 0, bytes32(uint256(1)));
        vm.prank(op1);
        vm.expectRevert(bytes("dup attestation"));
        svc.attest(id, 0, bytes32(uint256(2)), "");
    }

    function test_expiry_blocks_attest_and_marks_expired() public {
        StablecoinCollateralBVS.Action[] memory actions =
            new StablecoinCollateralBVS.Action [] (1);
        actions[0] = _action(target1, bytes4(0xBBBBBBBB), "", StablecoinCollateralBVS.MatchMode.NONE);

        address [] memory allow = new address[](0);
        uint256 id = _open(
            StablecoinCollateralBVS.CompletionMode.ANY,
            0, 0, 1, 1, // ttl=1s
            actions,
            allow
        );

        skip(2);

        vm.prank(op1);
        svc.attest(id, 0, bytes32(uint256(1)), "");
        

        // Read status
        StablecoinCollateralBVS.ReqStatus st= svc.checkRequestStatus(id);
        console.log(uint256(st));
        assertEq(uint256(st), uint256(StablecoinCollateralBVS.ReqStatus.Expired), "expired");
    }

    function test_cancel_request() public {
        StablecoinCollateralBVS.Action[] memory actions =
            new StablecoinCollateralBVS.Action [] (1);
        actions[0] = _action(target1, bytes4(0xCCCCCCCC), "", StablecoinCollateralBVS.MatchMode.NONE);

        address [] memory allow = new address[](0);
        uint256 id = _open(
            StablecoinCollateralBVS.CompletionMode.ALL,
            0, 0, 2, 1 days,
            actions,
            allow
        );

        vm.prank(owner);
        svc.cancelRequest(id, "change plan");

        (,,,,,,, StablecoinCollateralBVS.ReqStatus st,,,) = svc.requests(id);
        assertEq(uint256(st), uint256(StablecoinCollateralBVS.ReqStatus.Cancelled), "canceled");
    }

    function test_canFinalize_counts_and_required() public {
        StablecoinCollateralBVS.Action[] memory actions =
            new StablecoinCollateralBVS.Action [] (2);
        actions[0] = _action(target1, bytes4(0xAAAA0001), "", StablecoinCollateralBVS.MatchMode.NONE);
        actions[1] = _action(target2, bytes4(0xAAAA0002), "", StablecoinCollateralBVS.MatchMode.NONE);


        address [] memory allow = new address[](0);
        uint256 id = _open(
            StablecoinCollateralBVS.CompletionMode.ALL,
            0, 0, 2, 1 days,
            actions,
            allow
        );

        (bool ok0, uint16 req0, uint16[] memory counts0, uint256 sat0) = svc.canFinalize(id);
        assertFalse(ok0);
        assertEq(req0, 2);
        assertEq(counts0.length, 2);
        assertEq(counts0[0], 0);
        assertEq(counts0[1], 0);
        assertEq(sat0, 0);

        _attest(op1, id, 0, bytes32(uint256(1)));
        _attest(op2, id, 0, bytes32(uint256(2)));
        (bool ok1,, uint16[] memory counts1, uint256 sat1) = svc.canFinalize(id);
        assertFalse(ok1);
        assertEq(counts1[0], 2);
        assertEq(counts1[1], 0);
        assertEq(sat1, 1);

        _attest(op1, id, 1, bytes32(uint256(3)));
        _attest(op3, id, 1, bytes32(uint256(4)));
        (bool ok2,, uint16[] memory counts2, uint256 sat2) = svc.canFinalize(id);
        assertTrue(ok2);
        assertEq(counts2[0], 2);
        assertEq(counts2[1], 2);
        assertEq(sat2, 2);

        vm.prank(owner);
        svc.finalizeRequest(id);
    }

    function test_notifyUnsolicited_emits_for_active_op() public {
        vm.expectEmit(true, true, true, true, address(svc));
        emit StablecoinCollateralBVS.Unsolicited(
            uint64(block.chainid),
            op1,
            target1,
            bytes4(0xABCD0001),
            bytes32(uint256(0x1234)),
            hex"1122"
        );

        vm.prank(op1);
        svc.notifyUnsolicited(
            uint64(block.chainid),
            target1,
            bytes4(0xABCD0001),
            bytes32(uint256(0x1234)),
            hex"1122"
        );
    }
}
