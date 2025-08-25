// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {RelationshipV2} from "@satlayer/contracts/RelationshipV2.sol";
import {SLAYRegistryV2} from "@satlayer/contracts/SLAYRegistryV2.sol";
import {SLAYRouterV2} from "@satlayer/contracts/SLAYRouterV2.sol";
import {ISLAYRouterSlashingV2} from "@satlayer/contracts/interface/ISLAYRouterSlashingV2.sol";
import {ISLAYRegistryV2} from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract BVS is Ownable {
    SLAYRegistryV2 immutable registry;
    SLAYRouterV2 immutable router;
    mapping(int64 => address) requests;
    mapping(int64 => mapping(address => int64)) responses;

    event Requested(address indexed sender, int64 input);
    event Responded(address indexed operator, int64 indexed input, int64 indexed output);
    event OperatorRegistration(address indexed operator, bool indexed status);
    event SlashRequested(address indexed operator, ISLAYRouterSlashingV2.Payload params);
    event SlashEnabled(ISLAYRegistryV2.SlashParameter params);
    event SlashDisabled();

    error Unauthorized();
    error ResponseNotFound();
    error RequestNotFound();
    error InvalidChallenge();
    error AlreadyResponded();

    constructor(SLAYRouterV2 router_, SLAYRegistryV2 registry_, address owner) Ownable(owner) {
        registry = registry_;
        router = router_;

        registry.registerAsService("www.dsquaring.com", "Decentralized Squaring");
    }

    /**
     * A number is to be requested for squaring
     */
    function request(int64 num) external {
        requests[num] = msg.sender;
        emit Requested(msg.sender, num);
    }

    /**
     * An operator should square the requested number off-chain
     * and respond to it.
     */
    function respond(int64 input, int64 output) external {
        address operator = msg.sender;
        address requester = requests[input];

        if (requester == address(0)) {
            revert RequestNotFound();
        }

        RelationshipV2.Status registrationStatus = registry.getRelationshipStatus(address(this), operator);

        if (registrationStatus != RelationshipV2.Status.Active) {
            revert Unauthorized();
        }

        int64 prevOutput = responses[input][operator];

        if (prevOutput != 0) {
            revert AlreadyResponded();
        }

        responses[input][operator] = output;
        emit Responded(operator, input, output);
    }

    /**
     * Get a squared value responded by an operator for a particular input
     */
    function getResponse(int64 input, address operator) external view returns (int64) {
        return responses[input][operator];
    }

    /**
     * Anyone can challenge a squaring respond to a number by operator.
     * In the event of incorrect squaring of a number by an operator, slashing
     * lifecycle for the operator will be initiated.
     */
    function compute(int64 inp, address operator) external returns (bytes32) {
        int64 prevSquared = responses[inp][operator];

        if (prevSquared == 0) {
            revert ResponseNotFound();
        }

        int64 newSquared = _expensiveComputation(inp);

        if (prevSquared == newSquared) {
            revert InvalidChallenge();
        }

        responses[inp][operator] = newSquared;

        ISLAYRouterSlashingV2.Payload memory payload = ISLAYRouterSlashingV2.Payload({
            operator: operator,
            mbips: 1000,
            timestamp: uint32(block.timestamp),
            reason: "Invalid Proof"
        });

        emit SlashRequested(operator, payload);
        return router.requestSlashing(payload);
    }

    /**
     * Lock the slash collateral from targeted operator to SatLayer contract
     */
    function lockSlashing(bytes32 slashId) external onlyOwner {
        router.lockSlashing(slashId);
    }

    /**
     * Move the locked collateral from SatLayer contract to service designated address.
     */
    function finalizeSlashing(bytes32 slashId) external onlyOwner {
        router.finalizeSlashing(slashId);
    }

    /**
     * This function is an example of an expensive computation with
     * off-chain computing and on-chain objectively verifiable slashing.
     * You want to perform this off-chain to reduce gas costs.
     * When a malicious operator tries to cheat,
     * the on-chain verification can objectively verify the result by recomputing it on-chain.
     */
    function _expensiveComputation(int64 input) internal pure returns (int64) {
        return input * input;
    }

    /**
     * Register and recognized an address to be an operator the service.
     */
    function registerOperator(address operator) external onlyOwner {
        registry.registerOperatorToService(operator);
        emit OperatorRegistration(operator, true);
    }

    /**
     * Deregister an operator out of the service.
     */
    function deregisterOperator(address operator) external onlyOwner {
        registry.deregisterOperatorFromService(operator);
        emit OperatorRegistration(operator, false);
    }

    /**
     * Enable SatLayer integrated slashing.
     * If slashing is disabled, the slashing lifecycle in the event of malicious squaring challenge
     * to an operator will result in failure.
     */
    function enableSlashing(ISLAYRegistryV2.SlashParameter calldata params) external onlyOwner {
        registry.enableSlashing(params);
        emit SlashEnabled(params);
    }

    /**
     * Disable SatLayer integrated slashing.
     */
    function disableSlashing() external onlyOwner {
        registry.disableSlashing();
        emit SlashDisabled();
    }
}
