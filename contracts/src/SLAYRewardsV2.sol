// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import {SLAYBase} from "./SLAYBase.sol";
import {MerkleProof} from "./MerkleProof.sol";
import {ISLAYRewardsV2} from "./interface/ISLAYRewardsV2.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

/**
 * @title Rewards contract
 * @dev This contract serves as a contract for distributing rewards and claiming them.
 *
 * @custom:oz-upgrades-from src/SLAYBase.sol:SLAYBase
 */
contract SLAYRewardsV2 is
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable,
    SLAYBase,
    ISLAYRewardsV2
{
    using SafeERC20 for IERC20;

    /**
     * @dev Stores the merkle roots (previous and current) for each (service,token)'s distribution.
     * This is to prevent race conditions in the case of concurrent calls to distributeRewards and claimRewards.
     */
    mapping(address service => mapping(address token => DistributionRoots merkleRoots)) private _distributionRoots;

    /**
     * @dev Stores the rewards balances for each service and token received from {distributeRewards} function.
     */
    mapping(address service => mapping(address token => uint256 balance)) private _balances;

    /**
     * @dev Stores the total claimed rewards for each earner for a specific service and token.
     */
    mapping(address service => mapping(address token => mapping(address earner => uint256 totalClaimed))) private
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
    function getDistributionRoots(address service, address token) external view returns (DistributionRoots memory) {
        return _distributionRoots[service][token];
    }

    /// @inheritdoc ISLAYRewardsV2
    function getBalance(address service, address token) external view returns (uint256) {
        return _balances[service][token];
    }

    /// @inheritdoc ISLAYRewardsV2
    function getClaimedRewards(address service, address token, address earner) external view returns (uint256) {
        return _claimedRewards[service][token][earner];
    }

    /// @inheritdoc ISLAYRewardsV2
    function distributeRewards(address token, uint256 amount, bytes32 merkleRoot) external override whenNotPaused {
        require(merkleRoot != bytes32(0), "Merkle root cannot be empty");
        require(token != address(0), "Token address cannot be zero");

        // transfer the tokens from the caller to this contract if amount is greater than zero
        if (amount > 0) {
            SafeERC20.safeTransferFrom(IERC20(token), _msgSender(), address(this), amount);

            // update internal state
            _balances[_msgSender()][token] += amount;
        }

        // save distribution roots
        DistributionRoots storage roots = _distributionRoots[_msgSender()][token];
        roots.prevRoot = roots.currentRoot;
        roots.currentRoot = merkleRoot;

        emit RewardsDistributed(_msgSender(), token, amount, merkleRoot);
    }

    /// @inheritdoc ISLAYRewardsV2
    function claimRewards(ClaimableRewardProof calldata params) external override whenNotPaused {
        require(params.token != address(0), "Token address cannot be zero");
        require(params.amount > 0, "Amount must be greater than zero");
        require(params.merkleRoot != bytes32(0), "Merkle root cannot be empty");

        // check if merkle root either matches the current or previous root for the service
        DistributionRoots storage roots = _distributionRoots[params.service][params.token];
        if (params.merkleRoot != roots.currentRoot && params.merkleRoot != roots.prevRoot) {
            revert InvalidMerkleRoot(params.service, params.token, params.merkleRoot);
        }

        address earner = _msgSender();

        uint256 claimedAmount = _claimedRewards[params.service][params.token][earner];

        // check that amount is more than claimed amount
        if (params.amount <= claimedAmount) {
            revert AmountAlreadyClaimed(params.service, params.token, earner, params.amount);
        }

        uint256 amountToClaim;

        unchecked {
            // params.amount is asserted to be greater than claimedAmount, so we can safely calculate the amount to claim
            amountToClaim = params.amount - claimedAmount;
        }

        // check if the service has enough balance to cover the claim
        uint256 serviceBalance = _balances[params.service][params.token];
        if (serviceBalance < amountToClaim) {
            revert InsufficientBalance({service: params.service, token: params.token});
        }

        // verify the Merkle proof
        if (
            !_verifyMerkleProof(
                params.proof, params.merkleRoot, params.leafIndex, params.totalLeaves, earner, params.amount
            )
        ) {
            revert InvalidMerkleProof();
        }

        unchecked {
            // reduce (service,token) balance. serviceBalance >= amountToClaim is checked above
            _balances[params.service][params.token] = serviceBalance - amountToClaim;
        }

        // set the claimed rewards for the (service, token, earner) mapping to params.amount
        _claimedRewards[params.service][params.token][earner] = params.amount;

        // transfer the tokens to the recipient
        SafeERC20.safeTransfer(IERC20(params.token), params.recipient, amountToClaim);

        emit RewardsClaimed(params.service, params.token, earner, params.recipient, amountToClaim, params.merkleRoot);
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
     * The earner is represented as a checksum hex string to ensure that the address is in a consistent format with the rewards distribution file submitted by the service.
     *
     * @param earner The address of the earner.
     * @param amount The amount associated with the earner.
     * @return The hash of the leaf node.
     */
    function _leafHash(address earner, uint256 amount) internal pure returns (bytes32) {
        string memory earnerStringBytes = Strings.toChecksumHexString(earner);
        string memory amountStringBytes = Strings.toString(amount);
        return keccak256(abi.encodePacked(keccak256(abi.encodePacked(earnerStringBytes, amountStringBytes))));
    }
}
