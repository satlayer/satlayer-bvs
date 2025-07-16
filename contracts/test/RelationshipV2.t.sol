// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";

import {RelationshipV2} from "../src/RelationshipV2.sol";

contract RelationshipV2Test is Test {
    function test_objectDefault() external {
        RelationshipV2.Object memory rel;
        assertTrue(rel.status == RelationshipV2.Status.Inactive);
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
}
