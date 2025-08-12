// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {SLAYRegistryV2} from "@satlayer/contracts/SLAYRegistryV2.sol";
import { ISLAYRegistryV2 } from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";

/**
 * @title MultiSigWallet
 * @dev A basic multi-signature wallet contract.
 * This contract enables a group of owners to manage assets by requiring
 * a minimum number of confirmations for every transaction.
 *
 * It does not use a governance token for voting; instead, the owners
 * themselves vote by confirming a transaction.
 */
contract MultiSigFixedBVS is ReentrancyGuard {
    // --- State Variables ---

    // An array of the addresses that are owners of this wallet.
    address[] public owners;
    address public registryAddress;
    address public routerAddress;

    // A mapping to check if an address is an owner.
    mapping(address => bool) public isOwner;

    // The minimum number of confirmations required for a transaction to pass.
    uint256 public threshold;

    // A struct to store the details of a transaction proposal.
    struct Transaction {
        address destination; // The address the transaction is being sent to.
        uint256 value;       // The amount of Ether to send with the transaction.
        bytes data;          // The calldata for the transaction.
        bool executed;       // A flag to check if the transaction has been executed.
        uint256 confirmationsCount; // Number of confirmations received.
    }

    // A mapping from a transaction index to its details.
    mapping(uint256 => Transaction) public transactions;

    // A mapping to track which owners have confirmed a given transaction.
    mapping(uint256 => mapping(address => bool)) public confirmations;

    // A counter for the total number of transactions submitted.
    uint256 public transactionCount;

    // --- Events ---

    // Emitted when a new transaction is submitted.
    event Submission(uint256 indexed transactionId);

    // Emitted when an owner confirms a transaction.
    event Confirmation(address indexed owner, uint256 indexed transactionId);

    // Emitted when an owner revokes their confirmation.
    event Revocation(address indexed owner, uint256 indexed transactionId);

    // Emitted when a transaction is successfully executed.
    event Execution(uint256 indexed transactionId);

    // --- Modifiers ---

    // Restricts a function to be called only by an owner.
    modifier onlyOwners() {
        require(isOwner[msg.sender], "MultiSigWallet: not an owner");
        _;
    }

    // Ensures a transaction ID is valid.
    modifier transactionExists(uint256 _transactionId) {
        require(_transactionId < transactionCount, "MultiSigWallet: transaction does not exist");
        _;
    }

    // Ensures a transaction has not been executed yet.
    modifier notExecuted(uint256 _transactionId) {
        require(!transactions[_transactionId].executed, "MultiSigWallet: transaction already executed");
        _;
    }

    // --- Constructor ---

    /**
     * @dev Initializes the contract with a set of owners and a threshold.
     * @param _owners The list of addresses that will be the initial owners.
     * @param _threshold The number of confirmations required to execute a transaction.
     */
    constructor(address[] memory _owners, uint256 _threshold, address _registryAddress, address _routerAddress) payable {
        require(_owners.length > 0, "MultiSigWallet: owners list cannot be empty");
        require(_threshold > 0 && _threshold <= _owners.length, "MultiSigWallet: invalid threshold");

        for (uint256 i = 0; i < _owners.length; i++) {
            require(_owners[i] != address(0), "MultiSigWallet: owner address cannot be zero");
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
     * @dev Submits a new transaction proposal.
     * @param _destination The address of the contract or account to call.
     * @param _value The amount of Ether to send with the transaction.
     * @param _data The calldata for the transaction.
     * @return The ID of the submitted transaction.
     */
    function submitTransaction(
        address _destination,
        uint256 _value,
        bytes memory _data
    ) public onlyOwners returns (uint256 transactionId) {
        transactionId = transactionCount;
        transactions[transactionId] = Transaction({
            destination: _destination,
            value: _value,
            data: _data,
            executed: false,
            confirmationsCount: 0
        });
        transactionCount++;

        emit Submission(transactionId);

        // The submitter automatically confirms the transaction.
        confirmTransaction(transactionId);

        return transactionId;
    }

    /**
     * @dev Confirms a transaction by an owner.
     * @param _transactionId The ID of the transaction to confirm.
     */
    function confirmTransaction(
        uint256 _transactionId
    ) public onlyOwners transactionExists(_transactionId) notExecuted(_transactionId) {
        require(!confirmations[_transactionId][msg.sender], "MultiSigWallet: transaction already confirmed by this owner");

        confirmations[_transactionId][msg.sender] = true;
        transactions[_transactionId].confirmationsCount++;

        emit Confirmation(msg.sender, _transactionId);
    }

    /**
     * @dev Executes a transaction once the confirmation threshold has been met.
     * @param _transactionId The ID of the transaction to execute.
     */
    function executeTransaction(
        uint256 _transactionId
    ) public onlyOwners transactionExists(_transactionId) notExecuted(_transactionId) nonReentrant {
        require(transactions[_transactionId].confirmationsCount >= threshold, "MultiSigWallet: not enough confirmations");

        Transaction storage tx = transactions[_transactionId];
        tx.executed = true;

        // Use a low-level call to execute the transaction.
        (bool success, ) = tx.destination.call{value: tx.value}(tx.data);
        require(success, "MultiSigWallet: transaction execution failed");

        emit Execution(_transactionId);
    }
}
