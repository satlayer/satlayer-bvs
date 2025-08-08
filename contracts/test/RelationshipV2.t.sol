// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {Test, console} from "forge-std/Test.sol";
import {Checkpoints} from "@openzeppelin/contracts/utils/structs/Checkpoints.sol";
import {RelationshipV2} from "../src/RelationshipV2.sol";

contract RelationshipV2Test is Test {
    Checkpoints.Trace224 internal _checkpoints;

    function test_objectDefault() public pure {
        RelationshipV2.Object memory rel;
        assertTrue(rel.status == RelationshipV2.Status.Inactive);
        assertEq(rel.slashParameterId, 0);
    }

    function test_statusUint8Values() public pure {
        assertEq(uint8(RelationshipV2.Status.Inactive), 0);
        assertEq(uint8(RelationshipV2.Status.Active), 1);
        assertEq(uint8(RelationshipV2.Status.OperatorRegistered), 2);
        assertEq(uint8(RelationshipV2.Status.ServiceRegistered), 3);

        RelationshipV2.Status status;
        assertTrue(status == RelationshipV2.Status.Inactive);
        assertEq(uint8(status), uint8(0));
    }

    function test_getKey() public {
        address service = vm.randomAddress();
        address operator = vm.randomAddress();

        vm.startSnapshotGas("RelationshipV2", "getKey(service,operator)");
        bytes32 key = RelationshipV2.getKey(service, operator);
        vm.stopSnapshotGas();
        bytes32 expectedKey = keccak256(abi.encodePacked(service, operator));
        assertEq(key, expectedKey);
    }

    function testFuzz_getKey(address service, address operator) public pure {
        bytes32 key1 = RelationshipV2.getKey(service, operator);
        bytes32 key2 = keccak256(abi.encodePacked(service, operator));

        assertEq(key1, key2, "Keys should be consistent across multiple calls");
    }

    function test_encode() public pure {
        RelationshipV2.Status status = RelationshipV2.Status.Active;
        uint32 slashParameterId = 123;

        uint224 encoded = RelationshipV2.encode(status, slashParameterId);

        // Status is stored in the first 8 bits
        assertEq(uint8(encoded), uint8(status));

        // SlashParameterId is stored after the first 8 bits
        assertEq(uint32(encoded >> 8), slashParameterId);
    }

    function test_decode() public pure {
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

    function test_upperLookup() external {
        RelationshipV2.Object memory obj1 =
            RelationshipV2.Object({status: RelationshipV2.Status.Active, slashParameterId: 100});
        RelationshipV2.push(_checkpoints, 1000, obj1);

        RelationshipV2.Object memory obj2 =
            RelationshipV2.Object({status: RelationshipV2.Status.OperatorRegistered, slashParameterId: 200});
        RelationshipV2.push(_checkpoints, 2000, obj2);

        RelationshipV2.Object memory obj3 =
            RelationshipV2.Object({status: RelationshipV2.Status.ServiceRegistered, slashParameterId: 300});
        RelationshipV2.push(_checkpoints, 3000, obj3);

        // Test exact timestamp lookup
        RelationshipV2.Object memory result1 = RelationshipV2.upperLookup(_checkpoints, 2000);
        assertEq(result1.slashParameterId, obj2.slashParameterId);
        assertEq(uint8(result1.status), uint8(obj2.status));

        // Test timestamp between checkpoints (should return the prev checkpoint)
        RelationshipV2.Object memory result2 = RelationshipV2.upperLookup(_checkpoints, 1500);
        assertEq(result2.slashParameterId, obj1.slashParameterId);
        assertEq(uint8(result2.status), uint8(obj1.status));

        // Test timestamp before all checkpoints (should return no checkpoint aka default)
        RelationshipV2.Object memory result3 = RelationshipV2.upperLookup(_checkpoints, 500);
        assertEq(result3.slashParameterId, 0);
        assertEq(uint8(result3.status), uint8(0));

        // Test timestamp after all checkpoints (should return the last checkpoint)
        RelationshipV2.Object memory result4 = RelationshipV2.upperLookup(_checkpoints, 4000);
        assertEq(result4.slashParameterId, obj3.slashParameterId);
        assertEq(uint8(result4.status), uint8(obj3.status));
    }
}
