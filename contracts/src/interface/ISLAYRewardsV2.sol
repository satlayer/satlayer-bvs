// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

/**
 * @title Rewards Interface
 * @dev Interface for the SLAYRewardsV2 contract, which handles the distribution and claiming of rewards.
 */
interface ISLAYRewardsV2 {
    /*//////////////////////////////////////////////////////////////
                                ERRORS
    //////////////////////////////////////////////////////////////*/

    /// @dev Error thrown when an invalid Merkle root is provided for a service and token pair.
    error InvalidMerkleRoot(address service, address token, bytes32 merkleRoot);

    /// @dev Error thrown when an earner attempts to claim an amount that has already been claimed.
    error AmountAlreadyClaimed(address service, address token, address earner, uint256 amount);

    /// @dev Error thrown when an invalid Merkle proof is provided during reward claiming.
    error InvalidMerkleProof();

    /// @dev Error thrown when a service has insufficient balance for a token to distribute rewards.
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

    /**
     * @title Claimable Reward Proof
     * @dev Contains all the necessary information to verify and process a reward claim.
     * This struct is used as an input parameter for the claimRewards function.
     */
    struct ClaimableRewardProof {
        /// @dev The address of the service from which rewards are being claimed.
        address service;
        /// @dev The address of the token being claimed.
        address token;
        /// @dev The amount of tokens to claim.
        uint256 amount;
        /// @dev The address that will receive the claimed tokens.
        address recipient;
        /// @dev The Merkle root of the distribution from which the claim is made.
        bytes32 merkleRoot;
        /// @dev The Merkle proof verifying the claim's inclusion in the distribution.
        bytes32[] proof;
        /// @dev The index of the leaf in the Merkle tree.
        uint32 leafIndex;
        /// @dev The total number of leaves in the Merkle tree.
        uint32 totalLeaves;
    }

    /**
     * @title Distribution Roots
     * @dev Stores the previous and current Merkle roots for a service-token pair.
     * This allows for a transition period where claims can be made against either root.
     */
    struct DistributionRoots {
        /// @dev The previous Merkle root for the distribution.
        bytes32 prevRoot;
        /// @dev The current Merkle root for the distribution.
        bytes32 currentRoot;
    }

    /*//////////////////////////////////////////////////////////////
                                FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /**
     * @notice Returns the current and previous Merkle roots for a (service,token) pair.
     * @param service The address of the service.
     * @param token The address of the token.
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
     * When the {amount} is 0, the function will essentially only update the Merkle root without any token transfer.
     * Anybody can distribute rewards, this is not limited to the service itself to allow for flexibility in reward distribution.
     *
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
