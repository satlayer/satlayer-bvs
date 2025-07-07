// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

import {ISLAYRegistry} from "../src/interface/ISLAYRegistry.sol";
import {SLAYRouter} from "../src/SLAYRouter.sol";
import {Relationship} from "../src/Relationship.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYRegistryTest is Test, TestSuite {
    address public immutable service = makeAddr("service");
    address public immutable operator = makeAddr("operator");

    function setUp() public override {
        TestSuite.setUp();
    }

    function test_paused() public {
        vm.prank(owner);
        registry.pause();

        assertTrue(registry.paused());
    }

    function test_pausedOnlyOwnerError() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        registry.pause();
    }

    function test_unpausedOnlyOwnerError() public {
        vm.prank(owner);
        registry.pause();

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        registry.unpause();
    }

    function test_RegisterAsService() public {
        vm.expectEmit();
        emit ISLAYRegistry.ServiceRegistered(address(this));

        vm.expectEmit();
        emit ISLAYRegistry.MetadataUpdated(address(this), "https://service.example.com", "Service Name");

        registry.registerAsService("https://service.example.com", "Service Name");
        assertTrue(registry.isService(address(this)), "Service should be registered");
    }

    function test_RegisterAsService_AlreadyRegistered() public {
        registry.registerAsService("http://uri.com", "Name");

        vm.expectRevert("Already registered");
        registry.registerAsService("http://uri2.com", "Name 2");
    }

    function test_RegisterAsService_EmptyMetadata() public {
        vm.prank(vm.randomAddress());
        registry.registerAsService("https://service.example.com", "");

        vm.prank(vm.randomAddress());
        registry.registerAsService("", "Service Name");

        vm.prank(vm.randomAddress());
        registry.registerAsService("", "");
    }

    function test_RegisterAsService_Paused() public {
        vm.prank(owner);
        registry.pause();

        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        registry.registerAsService("https://service.example.com", "Service Name");
    }

    function test_RegisterAsOperator() public {
        vm.expectEmit();
        emit ISLAYRegistry.OperatorRegistered(address(this));

        vm.expectEmit();
        emit ISLAYRegistry.MetadataUpdated(address(this), "https://operator.com", "Operator X");

        registry.registerAsOperator("https://operator.com", "Operator X");
        assertTrue(registry.isOperator(address(this)), "Operator should be registered");
    }

    function test_RegisterAsOperator_AlreadyRegistered() public {
        registry.registerAsOperator("http://operator.com", "Operator X");

        vm.expectRevert("Already registered");
        registry.registerAsOperator("http://operating.com", "Operator X2");
    }

    function test_RegisterAsOperator_EmptyMetadata() public {
        vm.prank(vm.randomAddress());
        registry.registerAsOperator("https://operator.com", "");

        vm.prank(vm.randomAddress());
        registry.registerAsOperator("", "Operator X");

        vm.prank(vm.randomAddress());
        registry.registerAsOperator("", "");
    }

    function test_RegisterAsOperator_Paused() public {
        vm.prank(owner);
        registry.pause();

        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        registry.registerAsOperator("https://operator.com", "Operator X");
    }

    /**
     * Tests the full registration flow initiated by the service.
     * 1. Service registers the operator (status -> ServiceRegistered).
     * 2. Operator registers the service (status -> Active).
     */
    function test_full_ServiceInitiated() public {
        {
            vm.prank(service);
            registry.registerAsService("service.com", "Service A");
            vm.prank(operator);
            registry.registerAsOperator("operator.com", "Operator X");
        }

        // Step 1: Service registers operator
        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRegistry.RelationshipUpdated(service, operator, Relationship.Status.ServiceRegistered, 0);
        registry.registerOperatorToService(operator);

        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.ServiceRegistered),
            "Status should be ServiceRegistered"
        );

        // Step 2: Operator accept and register to the service
        vm.prank(operator);
        vm.expectEmit();
        emit ISLAYRegistry.RelationshipUpdated(service, operator, Relationship.Status.Active, 0);
        registry.registerServiceToOperator(service);
        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.Active),
            "Status should be Active"
        );
    }

    /**
     * Tests the full relationship flow initiated by the operator.
     * 1. Operator registers the service (status -> OperatorRegistered).
     * 2. Service registers the operator (status -> Active).
     */
    function test_full_OperatorInitiated() public {
        {
            vm.prank(service);
            registry.registerAsService("service.com", "Service A");
            vm.prank(operator);
            registry.registerAsOperator("operator.com", "Operator X");
        }

        // Step 1: Operator registers service
        vm.prank(operator);
        vm.expectEmit();
        emit ISLAYRegistry.RelationshipUpdated(service, operator, Relationship.Status.OperatorRegistered, 0);
        registry.registerServiceToOperator(service);

        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.OperatorRegistered),
            "Status should be OperatorRegistered"
        );

        // Step 2: Service accept and register operator
        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRegistry.RelationshipUpdated(service, operator, Relationship.Status.Active, 0);
        registry.registerOperatorToService(operator);

        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.Active),
            "Status should be Active"
        );
    }

    function test_Register_Paused() public {
        {
            vm.prank(service);
            registry.registerAsService("service.com", "Service A");
            vm.prank(operator);
            registry.registerAsOperator("operator.com", "Operator X");
        }

        vm.prank(owner);
        registry.pause();

        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        vm.prank(service);
        registry.registerOperatorToService(operator);

        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        vm.prank(operator);
        registry.registerServiceToOperator(service);
    }

    function test_DeregisterOperatorFromService() public {
        test_full_ServiceInitiated();

        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRegistry.RelationshipUpdated(service, operator, Relationship.Status.Inactive, 0);
        registry.deregisterOperatorFromService(operator);

        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.Inactive),
            "Status should be Inactive after deregistration"
        );
    }

    function test_DeregisterServiceFromOperator() public {
        test_full_OperatorInitiated();

        vm.prank(operator);
        vm.expectEmit();
        emit ISLAYRegistry.RelationshipUpdated(service, operator, Relationship.Status.Inactive, 0);
        registry.deregisterServiceFromOperator(service);

        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.Inactive),
            "Status should be Inactive after deregistration"
        );
    }

    function test_Deregister_Paused() public {
        test_full_ServiceInitiated();

        vm.prank(owner);
        registry.pause();

        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        vm.prank(service);
        registry.deregisterOperatorFromService(operator);

        vm.expectRevert(PausableUpgradeable.EnforcedPause.selector);
        vm.prank(operator);
        registry.deregisterServiceFromOperator(service);
    }

    function _advanceBlockBy(uint256 newHeight) internal {
        vm.roll(block.number + newHeight);
        vm.warp(block.timestamp + (12 * newHeight));
    }

    function test_getRelationshipStatusAt() public {
        {
            vm.prank(service);
            registry.registerAsService("service.com", "Service A");
            vm.prank(operator);
            registry.registerAsOperator("operator.com", "Operator X");
        }

        _advanceBlockBy(1);

        // Initial state is Inactive
        uint32 timeBeforeRegister = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, timeBeforeRegister)),
            uint256(Relationship.Status.Inactive),
            "Status should be Inactive because no prior history"
        );

        // Inactive at current time
        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.Inactive),
            "Status should be Inactive because no prior history"
        );

        _advanceBlockBy(1);

        // Service initiates registration
        vm.prank(service);
        registry.registerOperatorToService(operator);
        uint32 timeAfterRegister = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, timeBeforeRegister)),
            uint256(Relationship.Status.Inactive),
            "Status should be Inactive prior to registration"
        );
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, timeAfterRegister)),
            uint256(Relationship.Status.ServiceRegistered),
            "Status should be ServiceRegistered"
        );
        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.ServiceRegistered),
            "Status should be ServiceRegistered at current time"
        );

        _advanceBlockBy(100);

        // Check previous block status
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, timeBeforeRegister)),
            uint256(Relationship.Status.Inactive),
            "Status should still be Inactive"
        );

        // Operator completes registration
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        uint32 timeAfterActive = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, timeAfterActive)),
            uint256(Relationship.Status.Active),
            "Status should be Active"
        );
        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.Active),
            "Status should be Active at current time"
        );

        // Check all previous checkpoint
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, 0)),
            uint256(Relationship.Status.Inactive),
            "Status should be Inactive at timestamp 0"
        );
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, timeBeforeRegister)),
            uint256(Relationship.Status.Inactive),
            "Status should be Inactive before registration"
        );
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, timeAfterRegister)),
            uint256(Relationship.Status.ServiceRegistered),
            "Status should be ServiceRegistered after registration"
        );
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, timeAfterActive)),
            uint256(Relationship.Status.Active),
            "Status should be Active after mutual registration"
        );
        assertEq(
            uint256(registry.getRelationshipStatusAt(service, operator, uint32(block.timestamp + 1000000000))),
            uint256(Relationship.Status.Active),
            "Status should be Active in the future"
        );
        assertEq(
            uint256(registry.getRelationshipStatus(service, operator)),
            uint256(Relationship.Status.Active),
            "Status should be Active at current time"
        );
    }

    function test_Fail_UnregisteredService() public {
        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistry.ServiceNotFound.selector, address(this)));
        registry.registerOperatorToService(operator);
    }

    function test_Fail_UnregisteredOperator() public {
        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistry.OperatorNotFound.selector, address(this)));
        registry.registerServiceToOperator(service);
    }

    function test_Fail_RegisterOperatorToService_UnregisteredOperator() public {
        registry.registerAsService("https://service.eth", "Service A");

        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistry.OperatorNotFound.selector, address(operator)));
        registry.registerOperatorToService(operator);
    }

    function test_Fail_RegisterServiceToOperator_UnregisteredService() public {
        registry.registerAsOperator("https://operator.eth", "Operator X");

        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistry.ServiceNotFound.selector, address(service)));
        registry.registerServiceToOperator(service);
    }

    function test_Fail_RegisterOperatorToService_AlreadyActive() public {
        test_full_ServiceInitiated();

        vm.prank(service);
        vm.expectRevert("Already active");
        registry.registerOperatorToService(operator);
    }

    function test_Fail_RegisterServiceToOperator_AlreadyActive() public {
        test_full_ServiceInitiated();

        vm.prank(operator);
        vm.expectRevert("Already active");
        registry.registerServiceToOperator(service);
    }

    function test_WithdrawalDelay() public {
        uint32 newDelay = 8 days;
        // register operator
        vm.startPrank(operator);
        registry.registerAsOperator("https://operator.com", "Operator X");

        // if delay is not updated, it should be the default 7 days
        assertEq(registry.getWithdrawalDelay(operator), 7 days, "Default withdrawal delay should be 7 days");

        vm.expectEmit();
        emit ISLAYRegistry.WithdrawalDelayUpdated(operator, newDelay);

        // Set the withdrawal delay to 8 days
        registry.setWithdrawalDelay(newDelay);

        assertEq(registry.getWithdrawalDelay(operator), newDelay, "Withdrawal delay should be updated");
    }

    function test_Fail_SetWithdrawalDelay_NotOperator() public {
        vm.startPrank(operator);

        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistry.OperatorNotFound.selector, operator));
        registry.setWithdrawalDelay(7 days);
    }

    function test_Fail_SetWithdrawalDelay_LessThanDefault() public {
        vm.startPrank(operator);
        registry.registerAsOperator("https://operator.com", "Operator X");

        vm.expectRevert("Delay must be at least more than or equal to 7 days");
        registry.setWithdrawalDelay(7 days - 1);

        vm.expectRevert("Delay must be at least more than or equal to 7 days");
        registry.setWithdrawalDelay(0);
    }

    function test_service_enableSlashing() public {
        vm.prank(service);
        registry.registerAsService("service.com", "Service A");

        address destination = vm.randomAddress();

        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRegistry.SlashParameterUpdated(service, destination, 100_000, 3600);
        registry.enableSlashing(
            ISLAYRegistry.SlashParameter({destination: destination, maxMbips: 100000, resolutionWindow: 3600})
        );

        ISLAYRegistry.SlashParameter memory param = registry.getSlashParameter(service);

        assertEq(param.destination, destination);
        assertEq(param.maxMbips, 100_000);
        assertEq(param.resolutionWindow, 3600);
    }

    function test_service_enableSlashing_notService() public {
        address notService = vm.randomAddress();
        vm.prank(notService);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistry.ServiceNotFound.selector, notService));

        registry.enableSlashing(
            ISLAYRegistry.SlashParameter({destination: vm.randomAddress(), maxMbips: 10000, resolutionWindow: 100000})
        );
    }

    function test_service_enableSlashing_revert_conditions() public {
        vm.startPrank(service);

        registry.registerAsService("service.com", "Service A");

        vm.expectRevert("destination!=0");
        registry.enableSlashing(
            ISLAYRegistry.SlashParameter({destination: address(0), maxMbips: 10000, resolutionWindow: 100000})
        );

        vm.expectRevert("maxMbips!=0");
        registry.enableSlashing(
            ISLAYRegistry.SlashParameter({destination: vm.randomAddress(), maxMbips: 0, resolutionWindow: 100000})
        );

        vm.expectRevert("maxMbips!=>10000000");
        registry.enableSlashing(
            ISLAYRegistry.SlashParameter({
                destination: vm.randomAddress(),
                maxMbips: 10_000_001,
                resolutionWindow: 100000
            })
        );

        vm.expectRevert("maxMbips!=>10000000");
        registry.enableSlashing(
            ISLAYRegistry.SlashParameter({
                destination: vm.randomAddress(),
                maxMbips: 15_000_000,
                resolutionWindow: 100000
            })
        );
    }

    function test_enableSlashing_lifecycle() public {
        vm.prank(operator);
        registry.registerAsOperator("operator.com", "Operator X");

        vm.startPrank(service);
        registry.registerAsService("service.com", "Service A");
        registry.enableSlashing(
            ISLAYRegistry.SlashParameter({destination: vm.randomAddress(), maxMbips: 100_000, resolutionWindow: 3600})
        );
        registry.registerOperatorToService(operator);
        vm.stopPrank();

        vm.prank(operator);
        registry.registerServiceToOperator(service);

        vm.prank(operator);
        // TODO(fuxingloh): check emit
        registry.enableSlashing(service);
    }
}
