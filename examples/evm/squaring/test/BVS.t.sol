// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "@satlayer/contracts/tests/TestSuiteV2.sol";
import {BVS} from "../src/BVS.sol";
import {ISLAYRouterSlashingV2} from "@satlayer/contracts/interface/ISLAYRouterSlashingV2.sol";
import {ISLAYRegistryV2} from "@satlayer/contracts/interface/ISLAYRegistryV2.sol";

contract BVSTest is Test, TestSuiteV2 {
    BVS public service;
    address public immutable operator = makeAddr("operator");

    function setUp() public override {
        TestSuiteV2.setUp();
        vm.prank(owner);
        service = new BVS(router, registry, owner);

        vm.startPrank(operator);
        registry.registerAsOperator("www.uri.com", "A name");
        registry.registerServiceToOperator(address(service));
        registry.setWithdrawalDelay(10 days);
        vm.stopPrank();

        vm.prank(owner);
        service.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: address(service), maxMbips: 100_000, resolutionWindow: 10 days})
        );
    }

    function test_request_respond() public {
        vm.prank(owner);
        service.registerOperator(operator);

        address anyone = makeAddr("anyone");
        vm.prank(anyone);
        service.request(2);

        vm.prank(operator);
        _advanceBlockBy(10);
        service.respond(2, 4);

        int64 out = service.getResponse(2, operator);
        assert(out == 4);
    }

    function test_invalid_proof_being_requested_slashing() public returns (bytes32) {
        _advanceBlockBy(100_000_000);

        vm.prank(owner);
        service.registerOperator(operator);

        address anyone = makeAddr("anyone");
        vm.prank(anyone);
        service.request(2);

        _advanceBlockBy(10);

        // wrong output
        vm.prank(operator);
        service.respond(2, 8);

        int64 out = service.getResponse(2, operator);

        // assert that wrong output got into the contract states.
        assert(out == 8);

        _advanceBlockBy(10);

        vm.prank(anyone);
        bytes32 slashId = service.compute(2, operator);

        vm.prank(anyone);
        ISLAYRouterSlashingV2.Request memory pendingSlashRequest =
            router.getPendingSlashingRequest(address(service), operator);
        ISLAYRouterSlashingV2.Request memory slashRequestById = router.getSlashingRequest(slashId);

        assert(pendingSlashRequest.operator == slashRequestById.operator);

        return slashId;
    }
}
