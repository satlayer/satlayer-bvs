// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

interface ISLAYRewardsV2 {
    /*//////////////////////////////////////////////////////////////
                                ERRORS
    //////////////////////////////////////////////////////////////*/

    error InvalidMerkleRoot(address service, address token, bytes32 merkleRoot);

    error AmountAlreadyClaimed(address service, address token, address earner, uint256 amount);

    error InvalidMerkleProof();

    error InsufficientBalance(address service, address token);

    /*//////////////////////////////////////////////////////////////
                                EVENTS
    //////////////////////////////////////////////////////////////*/

    /**
     * @notice Emitted when rewards are distributed by service.
     * @param service The address of the service distributing rewards.
     * @param token The address of the token being distributed.
     * @param amount The total amount of tokens distributed.
     * @param merkleRoot The Merkle root of the distribution.
     */
    event RewardsDistributed(
        address indexed service, address indexed token, uint256 amount, bytes32 indexed merkleRoot
    );

    /**
     * @notice Emitted when rewards are claimed by an earner.
     * @param service The address of the service from which rewards are claimed.
     * @param token The address of the token being claimed.
     * @param earner The address of the earner claiming rewards.
     * @param recipient The address receiving the claimed rewards.
     * @param amount The amount of tokens claimed.
     * @param merkleRoot The Merkle root of the distribution from which the claim is made.
     */
    event RewardsClaimed(
        address indexed service,
        address indexed token,
        address indexed earner,
        address recipient,
        uint256 amount,
        bytes32 merkleRoot
    );

    struct ClaimableRewardProof {
        address service;
        address token;
        uint256 amount;
        address recipient;
        bytes32 merkleRoot;
        bytes32[] proof;
        uint32 leafIndex;
        uint32 totalLeaves;
    }

    struct DistributionRoots {
        bytes32 prevRoot;
        bytes32 currentRoot;
    }

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @notice Returns the current and previous Merkle roots for a (service,token) pair.
     * @param service The address of the service.
     * @return DistributionRoots containing the previous and current Merkle roots.
     */
    function getDistributionRoots(address service, address token) external view returns (DistributionRoots memory);

    /**
     * @notice Returns the balance of a service for a specific token.
     * @param service The address of the service.
     * @param token The address of the token.
     * @return The balance of the service for the specified token.
     */
    function getBalance(address service, address token) external view returns (uint256);

    /**
     * @notice Returns the total claimed rewards for a specific service, token, and earner.
     * @param service The address of the service.
     * @param token The address of the token.
     * @param earner The address of the earner.
     * @return The total amount of claimed rewards for the specified service, token, and earner.
     */
    function getClaimedRewards(address service, address token, address earner) external view returns (uint256);

    /**
     * @notice Distributes rewards from a service to earners using a Merkle tree.
     * @dev Service needs to ensure proper allowance is made for the contract to transfer tokens.
     * @param token The address of the token to distribute.
     * @param amount The amount of tokens to distribute.
     * @param merkleRoot The Merkle root of the distribution.
     */
    function distributeRewards(address token, uint256 amount, bytes32 merkleRoot) external;

    /**
     * @notice Claims rewards for an earner for a specific service and token using merkle proof.
     * @dev The function checks the Merkle proof, updates the claimed rewards and send the tokens to the recipient.
     * @param params The parameters containing service, token, amount, recipient, merkleRoot, proof, leafIndex, and totalLeaves.
     */
    function claimRewards(ClaimableRewardProof calldata params) external;
}
