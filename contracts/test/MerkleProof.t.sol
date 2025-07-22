// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

import {Test, console} from "forge-std/Test.sol";
import {MerkleProof} from "../src/MerkleProof.sol";

contract MerkleProofTest is Test {
    // Helper function to simulate off-chain Merkle tree building
    // Mimics the non-commutative hashing logic: Hash(left || right)
    function _hashPair(bytes32 left, bytes32 right) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(left, right));
    }

    // Helper to generate a Merkle tree and return the root and proofs
    // This function is for testing purposes, simulating an off-chain process.
    function _generateMerkleTreeAndProofs(bytes32[] memory leaves)
        internal
        pure
        returns (bytes32 root, bytes32[][] memory allProofs)
    {
        uint256 numLeaves = leaves.length;
        require(numLeaves > 0, "No leaves provided");

        allProofs = new bytes32[][](numLeaves);

        // Pad leaves to the next power of 2 if necessary
        uint256 paddedNumLeaves = 1;
        while (paddedNumLeaves < numLeaves) {
            paddedNumLeaves *= 2;
        }

        bytes32[] memory currentLayer = new bytes32[](paddedNumLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            currentLayer[i] = leaves[i];
        }
        // Duplicate the last leaf for odd number of leaves if padding to power of 2
        // This is a common Merkle tree construction for non-power-of-2 leaves.
        for (uint256 i = numLeaves; i < paddedNumLeaves; i++) {
            currentLayer[i] = leaves[numLeaves - 1]; // Pad with duplicate of last leaf
        }

        bytes32[][] memory layers = new bytes32[][](Math.log2(paddedNumLeaves, Math.Rounding.Ceil) + 1);
        layers[0] = currentLayer;

        uint256 layerIdx = 0;
        while (currentLayer.length > 1) {
            bytes32[] memory nextLayer = new bytes32[](currentLayer.length / 2);
            for (uint256 i = 0; i < currentLayer.length; i += 2) {
                nextLayer[i / 2] = _hashPair(currentLayer[i], currentLayer[i + 1]);
            }
            layerIdx++;
            currentLayer = nextLayer;
            layers[layerIdx] = currentLayer;
        }

        root = currentLayer[0]; // The final root

        // Generate proofs for each leaf
        for (uint256 i = 0; i < numLeaves; i++) {
            uint256 currentLeafIndex = i;
            bytes32[] memory proofForLeaf = new bytes32[](layerIdx); // proof.length should be treeHeight

            for (uint256 j = 0; j < layerIdx; j++) {
                bytes32[] memory layer = layers[j];
                if (currentLeafIndex % 2 == 0) {
                    // Current node is left child
                    proofForLeaf[j] = layer[currentLeafIndex + 1]; // Sibling is right
                } else {
                    // Current node is right child
                    proofForLeaf[j] = layer[currentLeafIndex - 1]; // Sibling is left
                }
                currentLeafIndex /= 2; // Move to parent index
            }
            allProofs[i] = proofForLeaf;
        }
    }

    function test_verify_bbn() public pure {
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

    function test_verify_evm() public pure {
        bytes32 leaf = keccak256(
            abi.encodePacked(
                keccak256(abi.encodePacked("0x86d6Fda2f439537da03a5b76D5aE26412F4c4235200000000000000000"))
            )
        );

        bytes32[] memory proof = new bytes32[](3);
        proof[0] = bytes32(0xc5d11bcf5b13a6839acbf0f57fe1b202fe159e5b5b3bbbd3b9dd1a69e1aa84dc);
        proof[1] = bytes32(0x8d25a6cb91e258d097872c7e37477e311da5fcd048037a7d729d9eac13903882);
        proof[2] = bytes32(0x8a08f27e959995b62300cc7b9cdebb565e9ba6c0bfabf76c58da0c98ac378e81);

        bytes32 root = bytes32(abi.encodePacked(hex"2016f97ae135385b6942e4aa35c97bdcfdd599c9ddcd750868f8366173d58d3c"));

        assertEq(MerkleProof.verify(proof, root, leaf, 3, 5), true);
    }

    function test_verify_complex() public pure {
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

    function test_verify_largeTree_powerOf2Leaves_firstLeaf() public pure {
        uint256 numLeaves = 16; // A power of 2
        bytes32[] memory leaves = new bytes32[](numLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf", i));
        }

        (bytes32 root, bytes32[][] memory allProofs) = _generateMerkleTreeAndProofs(leaves);

        uint256 leafIndex = 0; // First leaf
        bytes32 leafToVerify = leaves[leafIndex];
        bytes32[] memory proof = allProofs[leafIndex];

        assertTrue(MerkleProof.verify(proof, root, leafToVerify, leafIndex, numLeaves));
    }

    function test_verify_largeTree_powerOf2Leaves_middleLeaf() public pure {
        uint256 numLeaves = 16; // A power of 2
        bytes32[] memory leaves = new bytes32[](numLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf", i));
        }

        (bytes32 root, bytes32[][] memory allProofs) = _generateMerkleTreeAndProofs(leaves);

        uint256 leafIndex = 7; // Middle leaf
        bytes32 leafToVerify = leaves[leafIndex];
        bytes32[] memory proof = allProofs[leafIndex];

        assertTrue(MerkleProof.verify(proof, root, leafToVerify, leafIndex, numLeaves));
    }

    function test_verify_largeTree_powerOf2Leaves_lastLeaf() public pure {
        uint256 numLeaves = 16; // A power of 2
        bytes32[] memory leaves = new bytes32[](numLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf", i));
        }

        (bytes32 root, bytes32[][] memory allProofs) = _generateMerkleTreeAndProofs(leaves);

        uint256 leafIndex = numLeaves - 1; // Last leaf
        bytes32 leafToVerify = leaves[leafIndex];
        bytes32[] memory proof = allProofs[leafIndex];

        assertTrue(MerkleProof.verify(proof, root, leafToVerify, leafIndex, numLeaves));
    }

    function test_verify_largeTree_notPowerOf2Leaves_firstLeaf() public pure {
        uint256 numLeaves = 13; // Not a power of 2
        bytes32[] memory leaves = new bytes32[](numLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf", i));
        }

        (bytes32 root, bytes32[][] memory allProofs) = _generateMerkleTreeAndProofs(leaves);

        uint256 leafIndex = 0; // First leaf
        bytes32 leafToVerify = leaves[leafIndex];
        bytes32[] memory proof = allProofs[leafIndex];

        assertTrue(MerkleProof.verify(proof, root, leafToVerify, leafIndex, numLeaves));
    }

    function test_verify_largeTree_notPowerOf2Leaves_middleLeaf() public pure {
        uint256 numLeaves = 13; // Not a power of 2
        bytes32[] memory leaves = new bytes32[](numLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf", i));
        }

        (bytes32 root, bytes32[][] memory allProofs) = _generateMerkleTreeAndProofs(leaves);

        uint256 leafIndex = 6; // Middle leaf
        bytes32 leafToVerify = leaves[leafIndex];
        bytes32[] memory proof = allProofs[leafIndex];

        assertTrue(MerkleProof.verify(proof, root, leafToVerify, leafIndex, numLeaves));
    }

    function test_verify_largeTree_notPowerOf2Leaves_lastLeaf() public pure {
        uint256 numLeaves = 13; // Not a power of 2
        bytes32[] memory leaves = new bytes32[](numLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf", i));
        }

        (bytes32 root, bytes32[][] memory allProofs) = _generateMerkleTreeAndProofs(leaves);

        uint256 leafIndex = numLeaves - 1; // Last leaf
        bytes32 leafToVerify = leaves[leafIndex];
        bytes32[] memory proof = allProofs[leafIndex];

        assertTrue(MerkleProof.verify(proof, root, leafToVerify, leafIndex, numLeaves));
    }

    function test_revert_verify_incorrectProofForLeaf() public pure {
        uint256 numLeaves = 8;
        bytes32[] memory leaves = new bytes32[](numLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf", i));
        }

        (bytes32 root, bytes32[][] memory allProofs) = _generateMerkleTreeAndProofs(leaves);

        uint256 leafIndex = 3; // Correct leaf
        bytes32 leafToVerify = leaves[leafIndex];
        bytes32[] memory proof = allProofs[leafIndex];

        // Tamper with the proof: change one of the sibling hashes
        if (proof.length > 0) {
            proof[0] = keccak256(abi.encodePacked("TAMPERED_HASH")); // Invalidate the first sibling
        } else {
            // For a single leaf tree, there's no proof array to tamper with directly.
            // This case won't be hit with numLeaves = 8
        }

        console.log("\nTest: Revert on incorrect proof (tampered hash)");
        assertFalse(MerkleProof.verify(proof, root, leafToVerify, leafIndex, numLeaves));
    }

    function test_revert_verify_nonExistentLeaf() public pure {
        uint256 numLeaves = 8;
        bytes32[] memory leaves = new bytes32[](numLeaves);
        for (uint256 i = 0; i < numLeaves; i++) {
            leaves[i] = keccak256(abi.encodePacked("leaf", i));
        }

        (bytes32 root, bytes32[][] memory allProofs) = _generateMerkleTreeAndProofs(leaves);

        bytes32 nonExistentLeaf = keccak256(abi.encodePacked("I am not in the tree"));
        uint256 leafIndex = 0; // The index doesn't strictly matter for non-existent leaf, but must be valid based on numLeaves
        bytes32[] memory proof = allProofs[leafIndex]; // Use a valid proof path from an existing leaf

        console.log("\nTest: Revert on non-existent leaf");
        assertFalse(MerkleProof.verify(proof, root, nonExistentLeaf, leafIndex, numLeaves));
    }
}
