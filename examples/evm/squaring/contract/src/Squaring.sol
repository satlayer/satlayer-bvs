// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { RelationshipV2 } from "@satlayer/RelationshipV2.sol";
import {SLAYRegistryV2} from "@satlayer/SLAYRegistryV2.sol";
import {SLAYRouterV2} from "@satlayer/SLAYRouterV2.sol";
import { Payload } from "@satlayer/interface/ISLAYRouterSlashingV2.sol";

contract Squaring {
    address owner;
    SLAYRegistryV2 immutable registry;
    SLAYRouterV2 immutable router;
    mapping(int64 => address) requests;
    mapping(int64 => mapping(address => int64)) responses;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event requested(address indexed sender, int64 number);

    error ZeroValueNotAllowed();
    error Unauthorized();
    error ResponseNotFound();
    error InvalidProof();

    modifier onlyOwner() {
        if (msg.sender != owner) revert Unauthorized();
        _;
    }

    constructor(SLAYRouterV2 router_, SLAYRegistryV2 registry_) {
        owner = msg.sender;
        registry = registry_;
        router = router_;

        registry.registerAsService("www.dsquaring.com", "Decentralized Squaring");
    }

    function request(int64 num) public {
        requests[num] = msg.sender;
        emit requested(msg.sender, num);
    }

    function compute(int64 inp, address operator) public {
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

    function _expensiveComputation(int64 input) private returns(int64){
        return input * input;
    }


function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "New owner is zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }
}
