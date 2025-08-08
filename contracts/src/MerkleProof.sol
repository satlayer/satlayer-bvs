// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.24;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

/**
 * @dev These functions deal with verification of Merkle Tree proofs, adapted from OpenZeppelin Contracts.
 *
 * OpenZeppelin's MerkleProof library only supports commutative hash functions, which would
 * require significant changes to our CLI and would also necessitate modifying our CosmWasm (CW) Merkle library to match.
 * Additionally, commutative Merkle trees are less secure than traditional (non-commutative) Merkle trees
 * unless specific conditions are strictly enforced, as noted in OpenZeppelin's own documentation.
 * To avoid these issues and maintain compatibility and security, we implemented this Merkle proof verification logic.
 * This implementation is adapted from OpenZeppelin's code,
 * but modified to support non-commutative hash functions tailored to our use case.
 */
library MerkleProof {
    error InvalidProofLength();

    /**
     * @dev this function verifies a Merkle proof for a given leaf, index, and total number of leaves.
     * It computes the Merkle root from the proof and checks if it matches the provided root.
     * @param proof The Merkle proof containing sibling hashes that are needed to compute the root.
     * @param root The expected Merkle root to verify against.
     * @param leaf The leaf node to verify inclusion in the Merkle tree.
     * @param index The index of the leaf in the Merkle tree.
     * @param totalLeaves The total number of leaves in the Merkle tree.
     * @return bool indicating whether the proof is valid or not.
     */
    function verify(bytes32[] memory proof, bytes32 root, bytes32 leaf, uint256 index, uint256 totalLeaves)
        internal
        pure
        returns (bool)
    {
        return processProof(proof, leaf, index, totalLeaves) == root;
    }

    /**
     * @dev this function processes a Merkle proof to compute the Merkle root from a given leaf and its index.
     * It also does sanity checks on the proof length and the total number of leaves to prevent second pre-image attacks.
     * This function assumes that the Merkle tree is built using the keccak256 hash function.
     * This function assumes that the node hash used is non-commutative, meaning that the order of the hashes matters.
     * @param proof The Merkle proof containing sibling hashes that are needed to compute the root.
     * @param leaf The leaf node to verify inclusion in the Merkle tree.
     * @param index The index of the leaf in the Merkle tree.
     * @param totalLeaves The total number of leaves in the Merkle tree.
     * @return The computed Merkle root.
     */
    function processProof(bytes32[] memory proof, bytes32 leaf, uint256 index, uint256 totalLeaves)
        internal
        pure
        returns (bytes32)
    {
        // treeHeight is log2(totalLeaves), so the proof length must match the height of the tree
        uint256 treeHeight = Math.log2(totalLeaves, Math.Rounding.Ceil);
        if (treeHeight != proof.length) revert InvalidProofLength();

        // hash the node with the proof to rebuild the Merkle root
        bytes32 computedHash = leaf;
        // in dynamic arrays, the first 32 bytes are the length of the array,
        for (uint256 i = 1; i <= proof.length; i++) {
            if (index % 2 == 0) {
                // if index is even, then computedHash is a left sibling
                assembly {
                    mstore(0x00, computedHash)
                    mstore(0x20, mload(add(proof, mul(i, 0x20))))
                    computedHash := keccak256(0x00, 0x40)
                    index := div(index, 2)
                }
            } else {
                // if index is odd, then computedHash is a right sibling
                assembly {
                    mstore(0x00, mload(add(proof, mul(i, 0x20))))
                    mstore(0x20, computedHash)
                    computedHash := keccak256(0x00, 0x40)
                    index := div(index, 2)
                }
            }
        }
        return computedHash;
    }
}
