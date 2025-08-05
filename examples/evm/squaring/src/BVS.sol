// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { RelationshipV2 } from "@satlayer/contracts/RelationshipV2.sol";
import {SLAYRegistryV2} from "@satlayer/contracts/SLAYRegistryV2.sol";
import {SLAYRouterV2} from "@satlayer/contracts/SLAYRouterV2.sol";
import { Payload } from "@satlayer/contracts/interface/ISLAYRouterSlashingV2.sol";
import { SlashParameter } from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";

contract Squaring {
    address owner;
    SLAYRegistryV2 immutable registry;
    SLAYRouterV2 immutable router;
    mapping(int64 => address) requests;
    mapping(int64 => mapping(address => int64)) responses;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event requested(address indexed sender, int64 number);
    event contractRegistered(address indexed thisContract);
    event operatorRegistration(address indexed status, bool indexed status);
    event slashEnabled(SlashParameter params);
    event slashDisabled();

    error ZeroValueNotAllowed();
    error Unauthorized();
    error ResponseNotFound();
    error RequestNotFound();
    error InvalidProof();
    error Responded();

    modifier onlyOwner() {
        if (msg.sender != owner) revert Unauthorized();
        _;
    }

    constructor(SLAYRouterV2 router_, SLAYRegistryV2 registry_) {
        owner = msg.sender;
        registry = registry_;
        router = router_;

        registry.registerAsService("www.dsquaring.com", "Decentralized Squaring");
        emit contractRegistered(this);
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

        RelationshipV2.Status registrationStatus = registry.getRelationshipStatus(this, operator);

        if (registrationStatus != RelationshipV2.Status.Active) {
            revert Unauthorized();
        }

        int64 out = responses[input][operator];

        if(out == 0){
            revert Responded();
        }

        responses[input][operator] = out;
    }

    function getResponse(int64 input, address operator) external view {
        return responses[input][operator];
    }

    function compute(int64 inp, address operator) external {
        int64 prevSquared = responses[inp][operator];

        if (prevSquared == 0) {
            revert ResponseNotFound();
        }

        int64 newSquared = _expensiveComputation(inp);

        if(prevSquared != newSquared){
            revert InvalidProof();
        }

        router.requestSlashing(Payload({
            operator: operator,
            mbips: 1000,
            timestamp: uint32(block.timestamp),
            reason: "Invalid Proof"
        }));

    }

    function _expensiveComputation(int64 input) internal returns(int64){
        return input * input;
    }

    function registerOperator(address operator) public onlyOwner {
        registry.registerOperatorToService(operator);
        emit operatorRegistration(operator, true);
    }

    function deregisterOperator(address operator) public onlyOwner {
        registry.deregisterOperatorFromService(operator);
        emit operatorRegistration(operator, false);
    }

    function enableSlashing(SlashParameter calldata params) public onlyOwner {
        registry.enableSlashing(params);
        emit slashEnabled(params);
    }

    function disableSlashing() public onlyOwner {
        registry.disableSlashing();
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "New owner is zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }
}
