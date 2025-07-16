// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";
import {RelationshipV2} from "../src/RelationshipV2.sol";

contract RelationshipV2Test is Test {
    Checkpoints.Trace224 internal _checkpoints;

    function test_objectDefault() external {
        RelationshipV2.Object memory rel;
        assertTrue(rel.status == RelationshipV2.Status.Inactive);
        assertEq(rel.slashParameterId, 0);
    }

    function test_statusUint8Values() external {
        assertEq(uint8(RelationshipV2.Status.Inactive), 0);
        assertEq(uint8(RelationshipV2.Status.Active), 1);
        assertEq(uint8(RelationshipV2.Status.OperatorRegistered), 2);
        assertEq(uint8(RelationshipV2.Status.ServiceRegistered), 3);

        RelationshipV2.Status status;
        assertTrue(status == RelationshipV2.Status.Inactive);
        assertEq(uint8(status), uint8(0));
    }

    function test_getKey() external {
        address service = address(0x1);
        address operator = address(0x2);

        bytes32 key = RelationshipV2.getKey(service, operator);
        bytes32 expectedKey = keccak256(abi.encodePacked(service, operator));

        assertEq(key, expectedKey);
    }

    function test_encode() external {
        RelationshipV2.Status status = RelationshipV2.Status.Active;
        uint32 slashParameterId = 123;

        uint224 encoded = RelationshipV2.encode(status, slashParameterId);

        // Status is stored in the first 8 bits
        assertEq(uint8(encoded), uint8(status));

        // SlashParameterId is stored after the first 8 bits
        assertEq(uint32(encoded >> 8), slashParameterId);
    }

    function test_decode() external {
        RelationshipV2.Status status = RelationshipV2.Status.OperatorRegistered;
        uint32 slashParameterId = 456;

        uint224 encoded = RelationshipV2.encode(status, slashParameterId);
        RelationshipV2.Object memory decoded = RelationshipV2.decode(encoded);

        assertEq(uint8(decoded.status), uint8(status));
        assertEq(decoded.slashParameterId, slashParameterId);
    }

    function test_pushAndLatest() external {
        uint32 timestamp = 1000;
        RelationshipV2.Object memory obj =
            RelationshipV2.Object({status: RelationshipV2.Status.Active, slashParameterId: 789});

        RelationshipV2.push(_checkpoints, timestamp, obj);
        RelationshipV2.Object memory latest = RelationshipV2.latest(_checkpoints);

        assertEq(uint8(latest.status), uint8(obj.status));
        assertEq(latest.slashParameterId, obj.slashParameterId);
    }

    function test_upperLookup() external pure {}
}
