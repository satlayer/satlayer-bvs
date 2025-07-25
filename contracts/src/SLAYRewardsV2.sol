// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import {SLAYBase} from "./SLAYBase.sol";
import {MerkleProof} from "./MerkleProof.sol";
import {ISLAYRewardsV2} from "./interface/ISLAYRewardsV2.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {ERC4626Upgradeable} from "@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC4626Upgradeable.sol";

/**
 * @title Rewards contract
 * @dev This contract serves as a contract for distributing rewards and claiming them.
 *
 * @custom:oz-upgrades-from src/SLAYBase.sol:SLAYBase
 */
contract SLAYRewardsV2 is SLAYBase, ISLAYRewardsV2 {
    using SafeERC20 for IERC20;

    /**
     * @dev Stores the merkle roots (previous and current) for each (provider,token)'s distribution.
     * This is to prevent race conditions in the case of concurrent calls to distributeRewards and claimRewards.
     */
    mapping(address provider => mapping(address token => DistributionRoots merkleRoots)) private _distributionRoots;

    /**
     * @dev Stores the rewards balances for each provider and token received from {distributeRewards} function.
     */
    mapping(address provider => mapping(address token => uint256 balance)) private _balances;

    /**
     * @dev Stores the total claimed rewards for each earner for a specific provider and token.
     */
    mapping(address provider => mapping(address token => mapping(address earner => uint256 totalClaimed))) private
        _claimedRewards;

    /**
     * @dev Cyclic parameters in the constructor are possible because an SLAYBase (initial base implementation)
     * is used for the initial deployment, after which all contracts are upgraded to their respective
     * implementations with immutable proxy addresses.
     *
     * This contract extends SLAYBase, which provides the initial owner and pause functionality.
     * SLAYBase.initialize() is called to set the initial owner of the contract.
     *
     * @custom:oz-upgrades-unsafe-allow constructor
     */
    constructor() {
        _disableInitializers();
    }

    /// @inheritdoc ISLAYRewardsV2
    function getDistributionRoots(address provider, address token) external view returns (DistributionRoots memory) {
        return _distributionRoots[provider][token];
    }

    /// @inheritdoc ISLAYRewardsV2
    function getBalance(address provider, address token) external view returns (uint256) {
        return _balances[provider][token];
    }

    /// @inheritdoc ISLAYRewardsV2
    function getClaimedRewards(address provider, address token, address earner) external view returns (uint256) {
        return _claimedRewards[provider][token][earner];
    }

    /// @inheritdoc ISLAYRewardsV2
    function distributeRewards(address token, uint256 amount, bytes32 merkleRoot) external override whenNotPaused {
        require(merkleRoot != bytes32(0), "Merkle root cannot be empty");
        require(token != address(0), "Token address cannot be zero");

        address provider = _msgSender();
        // transfer the tokens from the caller to this contract if amount is greater than zero
        if (amount > 0) {
            // Similar to ERC4626 implementation:
            // If asset() is ERC-777, `transferFrom` can trigger a reentrancy BEFORE the transfer happens through the
            // `tokensToSend` hook. On the other hand, the `tokenReceived` hook, that is triggered after the transfer,
            // which is assumed not malicious.
            SafeERC20.safeTransferFrom(IERC20(token), provider, address(this), amount);

            // Hence, we need to do the transfer before we += so that any reentrancy would happen before the
            // assets are transferred and before the shares are added.
            _balances[provider][token] += amount;
        }

        // save distribution roots
        DistributionRoots storage roots = _distributionRoots[provider][token];
        roots.prevRoot = roots.currentRoot;
        roots.currentRoot = merkleRoot;

        emit RewardsDistributed(provider, token, amount, merkleRoot);
    }

    /// @inheritdoc ISLAYRewardsV2
    function claimRewards(ClaimableRewardProof calldata params) external override whenNotPaused {
        require(params.token != address(0), "Token address cannot be zero");
        require(params.amount > 0, "Amount must be greater than zero");
        require(params.merkleRoot != bytes32(0), "Merkle root cannot be empty");

        // check if merkle root either matches the current or previous root for the provider
        DistributionRoots storage roots = _distributionRoots[params.provider][params.token];
        if (params.merkleRoot != roots.currentRoot && params.merkleRoot != roots.prevRoot) {
            revert InvalidMerkleRoot(params.provider, params.token, params.merkleRoot);
        }

        address earner = _msgSender();

        uint256 claimedAmount = _claimedRewards[params.provider][params.token][earner];

        // check that amount is more than claimed amount
        if (params.amount <= claimedAmount) {
            revert AmountAlreadyClaimed(params.provider, params.token, earner, params.amount);
        }

        uint256 amountToClaim;

        // asserted params.amount > claimedAmount above, so we can safely calculate the amount to claim
        unchecked {
            amountToClaim = params.amount - claimedAmount;
        }

        // check if the provider has enough balance to cover the claim
        uint256 providerBalance = _balances[params.provider][params.token];
        if (providerBalance < amountToClaim) {
            revert InsufficientBalance({provider: params.provider, token: params.token});
        }

        // verify the Merkle proof
        if (
            !_verifyMerkleProof(
                params.proof, params.merkleRoot, params.leafIndex, params.totalLeaves, earner, params.amount
            )
        ) {
            revert InvalidMerkleProof();
        }

        // providerBalance >= amountToClaim is asserted above
        unchecked {
            // reduce (provider,token) balance.
            _balances[params.provider][params.token] = providerBalance - amountToClaim;
        }

        // set the claimed rewards for the (provider, token, earner) mapping to params.amount
        _claimedRewards[params.provider][params.token][earner] = params.amount;

        // Transfer after the burn so that any reentrancy would happen after the
        // shares are burned and after the assets are transferred.
        SafeERC20.safeTransfer(IERC20(params.token), params.recipient, amountToClaim);

        emit RewardsClaimed(params.provider, params.token, earner, params.recipient, amountToClaim, params.merkleRoot);
    }

    /**
     * @dev Internal function to verify the Merkle proof.
     * @param proof The Merkle proof to verify.
     * @param root The Merkle root to verify against.
     * @param index The index of the leaf in the Merkle tree.
     * @param totalLeaves The total number of leaves in the Merkle tree.
     * @param earner The address of the earner.
     * @param amount The amount associated with the earner.
     * @return True if the proof is valid, false otherwise.
     */
    function _verifyMerkleProof(
        bytes32[] memory proof,
        bytes32 root,
        uint256 index,
        uint256 totalLeaves,
        address earner,
        uint256 amount
    ) internal pure returns (bool) {
        bytes32 leaf = _leafHash(earner, amount);
        return MerkleProof.verify(proof, root, leaf, index, totalLeaves);
    }

    /**
     * @dev Internal function to hash the leaf node.
     * The leaf is a hash of the earner's address and the amount.
     * This is done to ensure that the leaf is unique for each (earner, amount) pair.
     * The leaf is hashed using double keccak256.
     *
     * The earner and amount are converted to strings and then hashed to ensure that it conform with the tree generation code that is chain-agnostic.
     * This will also allow future expansion into multi control plane claiming, where the earner might not be an evm address.
     * The earner is represented as a checksum hex string to ensure that the address is in a consistent format with the rewards distribution file submitted by the provider.
     *
     * @param earner The address of the earner.
     * @param amount The amount associated with the earner.
     * @return The hash of the leaf node.
     */
    function _leafHash(address earner, uint256 amount) internal pure returns (bytes32) {
        string memory earnerStringBytes = Strings.toChecksumHexString(earner);
        string memory amountStringBytes = Strings.toString(amount);
        /// We don't use inline assembly for keccak256 for this hashing function,
        /// due to the minimal gas savings and it doesn't fit into scratch space.
        /// It's better to maintain readability and security of the code.
        /// forge-lint: disable-start(asm-keccak256)
        return keccak256(abi.encodePacked(keccak256(abi.encodePacked(earnerStringBytes, amountStringBytes))));
    }
}
