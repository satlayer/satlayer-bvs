// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

import {Test, console} from "forge-std/Test.sol";
import {MerkleProof} from "../src/MerkleProof.sol";

contract MerkleProofTest is Test {
    function test_verify() public {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("bbn1eywhap4hwzd3lpwee4hgt2rh0rjlsq6dqck894100000000000000000"))
            )
        );

        bytes32[] memory proof = new bytes32[](1);
        proof[0] = bytes32(abi.encodePacked(hex"614a2406c3e74dca5a75c5429158a486c9d0d9eb5efbea928cc309beb6b3fce6"));

        bytes32 root = bytes32(abi.encodePacked(hex"4b83dc8ecaa7a9d69ac8a7c12718eed8639e1ba1a1b30a51741ccfd020255cec"));

        assertEq(MerkleProof.verify(proof, root, leaf, 1, 2), true);
    }

    function test_verify_complex() public {
        bytes32 leaf = keccak256(
            abi.encodePacked(keccak256(abi.encodePacked("bbn1j8k7l6m5n4o3p2q1r0s9t8u7v6w5x4y3z2a1b600000000000000000")))
        );

        bytes32[] memory proof = new bytes32[](4);
        proof[0] = bytes32(abi.encodePacked(hex"c99659b6e1c7a2df5c6ce352241ef43152f1fed170d2fdc8d67ee2c47d9f26a5"));
        proof[1] = bytes32(abi.encodePacked(hex"662362a44a545cd42cdf7e9c1cfc7eb3b55ebb3af452e1bd516d7329f0f490fa"));
        proof[2] = bytes32(abi.encodePacked(hex"8a12e22ffa7163c5249a71f3df41704c2e2ccf8bab02dec02fc8db2740f51256"));
        proof[3] = bytes32(abi.encodePacked(hex"afb5ee202bbe624a5d933b1eda40f5bf6bcd6674dbf1af8eea698ae023c104fe"));

        bytes32 root = bytes32(abi.encodePacked(hex"1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4"));

        assertEq(MerkleProof.verify(proof, root, leaf, 7, 9), true);
    }

    function test_verify_complex2() public {
        bytes32 leaf = keccak256(
            abi.encodePacked(keccak256(abi.encodePacked("bbn1j8k7l6m5n4o3p2q1r0s9t8u7v6w5x4y3z2a1b600000000000000000")))
        );

        bytes32[] memory proof = new bytes32[](4);
        proof[0] = bytes32(abi.encodePacked(hex"c99659b6e1c7a2df5c6ce352241ef43152f1fed170d2fdc8d67ee2c47d9f26a5"));
        proof[1] = bytes32(abi.encodePacked(hex"662362a44a545cd42cdf7e9c1cfc7eb3b55ebb3af452e1bd516d7329f0f490fa"));
        proof[2] = bytes32(abi.encodePacked(hex"8a12e22ffa7163c5249a71f3df41704c2e2ccf8bab02dec02fc8db2740f51256"));
        proof[3] = bytes32(abi.encodePacked(hex"afb5ee202bbe624a5d933b1eda40f5bf6bcd6674dbf1af8eea698ae023c104fe"));

        bytes32 root = bytes32(abi.encodePacked(hex"1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4"));

        assertEq(MerkleProof.verify(proof, root, leaf, 7, 9), true);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function test_revert_processProof_wrongTotalLeaves() public {
        bytes32 leaf = keccak256(
            abi.encodePacked(keccak256(abi.encodePacked("bbn1j8k7l6m5n4o3p2q1r0s9t8u7v6w5x4y3z2a1b600000000000000000")))
        );

        bytes32[] memory proof = new bytes32[](4);
        proof[0] = bytes32(abi.encodePacked(hex"c99659b6e1c7a2df5c6ce352241ef43152f1fed170d2fdc8d67ee2c47d9f26a5"));
        proof[1] = bytes32(abi.encodePacked(hex"662362a44a545cd42cdf7e9c1cfc7eb3b55ebb3af452e1bd516d7329f0f490fa"));
        proof[2] = bytes32(abi.encodePacked(hex"8a12e22ffa7163c5249a71f3df41704c2e2ccf8bab02dec02fc8db2740f51256"));
        proof[3] = bytes32(abi.encodePacked(hex"afb5ee202bbe624a5d933b1eda40f5bf6bcd6674dbf1af8eea698ae023c104fe"));

        bytes32 root = bytes32(abi.encodePacked(hex"1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4"));

        // expect revert because the total number of leaves is not correct
        vm.expectRevert(MerkleProof.InvalidProofLength.selector);
        MerkleProof.verify(proof, root, leaf, 7, 8); // totalLeaves should be 9, but we pass 8
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function test_revert_processProof_wrongProofLength() public {
        bytes32 leaf = keccak256(
            abi.encodePacked(keccak256(abi.encodePacked("bbn1j8k7l6m5n4o3p2q1r0s9t8u7v6w5x4y3z2a1b600000000000000000")))
        );

        bytes32[] memory proof = new bytes32[](3);
        proof[0] = bytes32(abi.encodePacked(hex"c99659b6e1c7a2df5c6ce352241ef43152f1fed170d2fdc8d67ee2c47d9f26a5"));
        proof[1] = bytes32(abi.encodePacked(hex"662362a44a545cd42cdf7e9c1cfc7eb3b55ebb3af452e1bd516d7329f0f490fa"));
        proof[2] = bytes32(abi.encodePacked(hex"8a12e22ffa7163c5249a71f3df41704c2e2ccf8bab02dec02fc8db2740f51256"));

        bytes32 root = bytes32(abi.encodePacked(hex"1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4"));

        // expect revert because the proof length must be 4
        vm.expectRevert(MerkleProof.InvalidProofLength.selector);
        MerkleProof.verify(proof, root, leaf, 7, 9);
    }
}
