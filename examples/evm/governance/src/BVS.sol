// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {SLAYRegistryV2} from "@satlayer/contracts/SLAYRegistryV2.sol";
import { ISLAYRegistryV2 } from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";

/**
 * @title BVS
 * @dev A basic multi-signature wallet contract committe based execution.
 * This contract enables a group of owners to manage transactions by requiring
 * a minimum number of confirmations for every proposal.
 *
 * It does not use a governance token for voting; instead, the owners
 * themselves vote by confirming a proposal.
 */
contract BVS is ReentrancyGuard {
    // --- State Variables ---

    // An array of the addresses that are owners of this wallet.
    address[] public owners;
    address public registryAddress;
    address public routerAddress;

    // A mapping to check if an address is an owner.
    mapping(address => bool) public isOwner;

    // The minimum number of confirmations required for a proposal to pass.
    uint256 public threshold;

    // A struct to store the details of a proposal.
    struct Proposal {
        address destination; // The address the proposal is being sent to.
        uint256 value;       // The amount of Ether to send with the proposal.
        bytes data;          // The calldata for the proposal.
        bool executed;       // A flag to check if the proposal has been executed.
        uint256 confirmationsCount; // Number of confirmations received.
    }

    // A mapping from a proposal index to its details.
    mapping(uint256 => Proposal) public proposals;

    // A mapping to track which owners have confirmed a given proposal.
    mapping(uint256 => mapping(address => bool)) public confirmations;

    // A counter for the total number of proposals submitted.
    uint256 public proposalCount;

    // --- Events ---

    // Emitted when a new proposal is submitted.
    event Submission(uint256 indexed proposalId);

    // Emitted when an owner confirms a proposal.
    event Confirmation(address indexed owner, uint256 indexed proposalId);

    // Emitted when an owner revokes their confirmation.
    event Revocation(address indexed owner, uint256 indexed proposalId);

    // Emitted when a proposal is successfully executed.
    event Execution(uint256 indexed proposalId);

    // --- Modifiers ---

    // Restricts a function to be called only by an owner.
    modifier onlyOwners() {
        require(isOwner[msg.sender], "BVS: not an owner");
        _;
    }

    // Ensures a proposal ID is valid.
    modifier proposalExists(uint256 _proposalId) {
        require(_proposalId < proposalCount, "BVS: proposal does not exist");
        _;
    }

    // Ensures a proposal has not been executed yet.
    modifier notExecuted(uint256 _proposalId) {
        require(!proposals[_proposalId].executed, "BVS: proposal already executed");
        _;
    }

    // --- Constructor ---

    /**
     * @dev Initializes the contract with a set of owners and a threshold.
     * @param _owners The list of addresses that will be the initial owners.
     * @param _threshold The number of confirmations required to execute a proposal.
     */
    constructor(address[] memory _owners, uint256 _threshold, address _registryAddress, address _routerAddress) payable {
        require(_owners.length > 0, "BVS: owners list cannot be empty");
        require(_threshold > 0 && _threshold <= _owners.length, "BVS: invalid threshold");

        for (uint256 i = 0; i < _owners.length; i++) {
            require(_owners[i] != address(0), "BVS: owner address cannot be zero");
            isOwner[_owners[i]] = true;
            owners.push(_owners[i]);
        }
        threshold = _threshold;

        registryAddress = _registryAddress;
        routerAddress = _routerAddress;

        ISLAYRegistryV2(registryAddress).registerAsService("www.dao.com", "A governance based Bitcoin Validated Service");
    }

    // --- Public Functions ---

    /**
     * @dev Allows this contract to receive Ether.
     */
    receive() external payable {}

    /**
     * @dev Submits a new proposal.
     * @param _destination The address of the contract or account to call.
     * @param _value The amount of Ether to send with the proposal.
     * @param _data The calldata for the proposal.
     * @return proposalId The ID of the newly created proposal.
     */
    function submitProposal(
        address _destination,
        uint256 _value,
        bytes memory _data
    ) public onlyOwners returns (uint256 proposalId) {
        proposalId = proposalCount;
        proposals[proposalId] = Proposal({
            destination: _destination,
            value: _value,
            data: _data,
            executed: false,
            confirmationsCount: 0
        });
        proposalCount++;

        emit Submission(proposalId);

        // The submitter automatically confirms the proposal.
        confirmProposal(proposalId);

        return proposalId;
    }

    /**
     * @dev Confirms a proposal by an owner.
     * @param _proposalId The ID of the proposal to confirm.
     */
    function confirmProposal(
        uint256 _proposalId
    ) public onlyOwners proposalExists(_proposalId) notExecuted(_proposalId) {
        require(!confirmations[_proposalId][msg.sender], "BVS: proposal already confirmed by this owner");

        confirmations[_proposalId][msg.sender] = true;
        proposals[_proposalId].confirmationsCount++;

        emit Confirmation(msg.sender, _proposalId);
    }

    /**
     * @dev Executes a proposal once the confirmation threshold has been met.
     * @param _proposalId The ID of the proposal to execute.
     */
    function executeProposal(
        uint256 _proposalId
    ) public onlyOwners proposalExists(_proposalId) notExecuted(_proposalId) nonReentrant {
        require(proposals[_proposalId].confirmationsCount >= threshold, "BVS: not enough confirmations");

        Proposal storage p = proposals[_proposalId];
        p.executed = true;

        // Use a low-level call to execute the proposal and capture the return data.
        (bool success, bytes memory returnData) = p.destination.call{value: p.value}(p.data);

        if (!success) {
            // revert with the exact bytes returned by the callee
            assembly {
                revert(add(returnData, 0x20), mload(returnData))
            }
        }

        emit Execution(_proposalId);
    }
}
