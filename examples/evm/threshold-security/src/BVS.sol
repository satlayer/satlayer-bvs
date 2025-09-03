// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {RelationshipV2} from "@satlayer/contracts/RelationshipV2.sol";
import {ISLAYRegistryV2} from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";
import {ISLAYRouterV2} from "@satlayer/contracts/interface/ISLAYRouterV2.sol";
import {ISLAYOracle} from "@satlayer/contracts/extension/interface/ISLAYOracle.sol";
import {ISLAYRouterSlashingV2} from "@satlayer/contracts/interface/ISLAYRouterSlashingV2.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";

contract BVS is Ownable {
    ISLAYRouterV2 public immutable router;
    ISLAYRegistryV2 public immutable registry;
    ISLAYOracle public immutable oracle;

    uint64 public nextRequestId = 0;

    Threshold public threshold;
    mapping(uint64 requestId => uint64 requestData) public requests;
    mapping(uint64 requestId => mapping(address operators => uint128 response)) public responses;
    mapping(uint64 requestId => mapping(uint128 response => uint256 totalTVL)) public responsesTVL;
    mapping(uint64 requestId => bool finalized) public finalizedRequests;

    event Requested(address indexed sender, uint64 indexed requestId, uint64 input);
    event Responded(address indexed operator, uint64 indexed requestId, uint128 indexed output);
    event OperatorRegistration(address indexed operator, bool indexed status);
    event SlashRequested(address indexed operator, ISLAYRouterSlashingV2.Payload params);
    event SlashEnabled(ISLAYRegistryV2.SlashParameter params);
    event SlashDisabled();
    event Finalized(uint64 indexed requestId, uint128 indexed output);

    error Unauthorized();
    error ResponseNotFound();
    error RequestNotFound();
    error AlreadyResponded();
    error AlreadyFinalized();
    error ThresholdNotMet();

    /**
     * @dev Threshold struct to define the threshold value (in USD) and its exponent.
     * @param threshold The minimum value in USD before a request can be finalized.
     * @param exponent The number of decimal places of the threshold.
     */
    struct Threshold {
        uint256 threshold; // in exponent units (eg. 1000 means 10.00 if exponent is 2)
        uint8 exponent;  // decimal places of the threshold
    }

    constructor(address router_, address registry_, address oracle_, address owner, Threshold memory threshold_) Ownable(owner) {
        router = ISLAYRouterV2(router_);
        registry = ISLAYRegistryV2(registry_);
        oracle = ISLAYOracle(oracle_);

        registry.registerAsService("www.thresholdSquaring.com", "Threshold Squaring");

        threshold = threshold_;
    }

    /**
     * @dev only owner can request a number to be squared
     */
    function request(uint64 num) external onlyOwner returns (uint64) {
        uint64 requestId = nextRequestId;
        requests[requestId] = num;
        nextRequestId += 1;
        emit Requested(msg.sender, requestId, num);
        return requestId;
    }


    /**
     * @dev operator can respond with the squared number
     */
    function respond(uint64 requestId, uint128 output) external {
        if (finalizedRequests[requestId]) {
            revert AlreadyFinalized();
        }
        address operator = msg.sender;

        // ensure only active relationship between service (this) and operator
        RelationshipV2.Status registrationStatus = registry.getRelationshipStatus(address(this), operator);
        if (registrationStatus != RelationshipV2.Status.Active) {
            revert Unauthorized();
        }

        // ensure operator has not responded before
        uint128 prevOutput = responses[requestId][operator];
        if (prevOutput != 0) {
            revert AlreadyResponded();
        }

        responses[requestId][operator] = output;

        uint256 operatorTVL = oracle.getOperatorAUM(operator);
        // convert operatorTVL (in 1e18) to threshold unit
        uint256 operatorTVLConverted = Math.mulDiv(operatorTVL, 10**threshold.exponent, 1e18);

        responsesTVL[requestId][output] += operatorTVLConverted;

        emit Responded(operator, requestId, output);
    }

    /**
     * @dev called by anyone to finalize a request if threshold is met
     */
    function finalize(uint64 requestId, uint128 output) external {
        if (finalizedRequests[requestId]) {
            revert AlreadyFinalized();
        }

        // get totalTVL in threshold decimals
        uint256 totalTVL = responsesTVL[requestId][output];

        if (totalTVL < threshold.threshold) {
            revert ThresholdNotMet();
        }

        finalizedRequests[requestId] = true;
        emit Finalized(requestId, output);
    }

    /**
   * Get a squared value responded by an operator for a particular input
   */
    function getResponse(uint64 requestId, address operator) external view returns (uint128) {
        return responses[requestId][operator];
    }
}
