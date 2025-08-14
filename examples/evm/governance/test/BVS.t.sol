// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "./TestSuite.t.sol";
import {BVS} from "../src/BVS.sol";
import { ISLAYRouterSlashingV2 } from "@satlayer/contracts/interface/ISLAYRouterSlashingV2.sol";
import { ISLAYRegistryV2 } from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";
import { SLAYRegistryV2 } from "@satlayer/contracts/SLAYRegistryV2.sol";
import { RelationshipV2 } from "@satlayer/contracts/RelationshipV2.sol";

contract BVSTest is Test, TestSuiteV2 {
    BVS public service;
    address public immutable operator = makeAddr("operator");
    address public immutable member1 = makeAddr("member1");
    address public immutable member2 = makeAddr("member2");
    address public immutable member3 = makeAddr("member3");
    address public immutable member4 = makeAddr("member4");
    address public immutable member5 = makeAddr("member5");
    address[] public members = new address[](5);

    function setUp() public override {
        members[0] = member1;
        members[1] = member2;
        members[2] = member3;
        members[3] = member4;
        members[4] = member5;

        TestSuiteV2.setUp();
        vm.prank(member1);
        service = new BVS(members, 4, address(registry), address(router));


        vm.startPrank(operator);
        registry.registerAsOperator("www.uri.com","A name");
        registry.registerServiceToOperator(address(service));
        registry.setWithdrawalDelay(10 days);
        vm.stopPrank();

    }

    function test_registerOperator_quorum() public {
        bytes memory data = abi.encodeWithSelector(
            SLAYRegistryV2.registerOperatorToService.selector,
            operator
        );

        vm.prank(member1);
        uint256 transactionId = service.submitProposal(
            address(registry),
            0,
            data
        );

        for (uint256 i = 1; i < members.length - 1; i++) {
            vm.prank(members[i]);
            service.confirmProposal(transactionId);
        }

        vm.prank(member1);
        service.executeProposal(transactionId);


        RelationshipV2.Status status = registry.getRelationshipStatus(address(service), operator);
        assertEq(uint8(status), uint8(RelationshipV2.Status.Active));
    }

    function test_enableSlashing_quorum() public {
        bytes memory data = abi.encodeWithSelector(
            SLAYRegistryV2.enableSlashing.selector,
            ISLAYRegistryV2.SlashParameter({
                destination: address(service),
                maxMbips: 10_000_000,
                resolutionWindow: 7 days
            })
        );

        vm.prank(member1);
        uint256 id = service.submitProposal(
            address(registry),
            0,
            data
        );

        for (uint256 i = 1; i < members.length - 1; i++) {
            vm.prank(members[i]);
            service.confirmProposal(id);
        }

        vm.prank(member1);
        service.executeProposal(id);


        ISLAYRegistryV2.SlashParameter memory param = registry.getSlashParameter(address(service));
        assertEq(param.destination, address(service));
    }

}
