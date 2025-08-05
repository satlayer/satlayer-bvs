// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { RelationshipV2 } from "@satlayer/contracts/RelationshipV2.sol";
import {SLAYRegistryV2} from "@satlayer/contracts/SLAYRegistryV2.sol";
import {SLAYRouterV2} from "@satlayer/contracts/SLAYRouterV2.sol";
import { ISLAYRouterSlashingV2 } from "@satlayer/contracts/interface/ISLAYRouterSlashingV2.sol";
import { ISLAYRegistryV2 } from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";

contract BVS {
    address owner;
    address immutable registry;
    address immutable router;
    mapping(int64 => address) requests;
    mapping(int64 => mapping(address => int64)) responses;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event requested(address indexed sender, int64 number);
    event contractRegistered(address indexed thisContract);
    event operatorRegistration(address indexed operator, bool indexed status);
    event slashRequested(address indexed accusedOperator, ISLAYRouterSlashingV2.Payload params);
    event slashEnabled(ISLAYRegistryV2.SlashParameter params);
    event slashDisabled();

    error ZeroValueNotAllowed();
    error Unauthorized();
    error ResponseNotFound();
    error RequestNotFound();
    error invalidChallenge();
    error Responded();

    modifier onlyOwner() {
        if (msg.sender != owner) revert Unauthorized();
        _;
    }

    constructor(address router_, address registry_) {
        owner = msg.sender;
        registry = registry_;
        router = router_;

        ISLAYRegistryV2(registry).registerAsService("www.dsquaring.com", "Decentralized Squaring");
        emit contractRegistered(address(this));
    }

    function request(int64 num) external {
        requests[num] = msg.sender;
        emit requested(msg.sender, num);
    }

    function respond(int64 input, int64 output) external {
        address operator = msg.sender;
        address eoa = requests[input];

        if (eoa == address(0)) {
            revert RequestNotFound();
        }

        RelationshipV2.Status registrationStatus = ISLAYRegistryV2(registry).getRelationshipStatus(address(this), operator);

        if (registrationStatus != RelationshipV2.Status.Active) {
            revert Unauthorized();
        }

        int64 prevOutput = responses[input][operator];

        if(prevOutput != 0){
            revert Responded();
        }

        responses[input][operator] = output;
    }

    function getResponse(int64 input, address operator) external view returns(int64) {
        return responses[input][operator];
    }

    function compute(int64 inp, address operator) external returns(bytes32) {
        int64 prevSquared = responses[inp][operator];

        if (prevSquared == 0) {
            revert ResponseNotFound();
        }

        int64 newSquared = _expensiveComputation(inp);

        if(prevSquared == newSquared){
            revert invalidChallenge();
        }

        ISLAYRouterSlashingV2.Payload memory payload = ISLAYRouterSlashingV2.Payload({
            operator: operator,
            mbips: 1000,
            timestamp: uint32(block.timestamp),
            reason: "Invalid Proof"
        });

        emit slashRequested(operator, payload);
        return SLAYRouterV2(router).requestSlashing(payload);
    }

    function _expensiveComputation(int64 input) internal returns(int64){
        return input * input;
    }

    function registerOperator(address operator) public onlyOwner {
        SLAYRegistryV2(registry).registerOperatorToService(operator);
        emit operatorRegistration(operator, true);
    }

    function deregisterOperator(address operator) public onlyOwner {
        SLAYRegistryV2(registry).deregisterOperatorFromService(operator);
        emit operatorRegistration(operator, false);
    }

    function enableSlashing(ISLAYRegistryV2.SlashParameter calldata params) public onlyOwner {
        SLAYRegistryV2(registry).enableSlashing(params);
        emit slashEnabled(params);
    }

    function disableSlashing() public onlyOwner {
        SLAYRegistryV2(registry).disableSlashing();
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "New owner is zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }
}
