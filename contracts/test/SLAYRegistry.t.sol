// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import {SLAYRegistry} from "../src/SLAYRegistry.sol";
import {SLAYRouter} from "../src/SLAYRouter.sol";
import {TestSuite} from "./TestSuite.sol";

contract SLAYRegistryTest is Test, TestSuite {
    address public immutable service = makeAddr("service");
    address public immutable operator = makeAddr("operator");

    function setUp() public override {
        TestSuite.setUp();
    }

    function test_RegisterAsService() public {
        vm.expectEmit();
        emit SLAYRegistry.ServiceRegistered(address(this));

        vm.expectEmit();
        emit SLAYRegistry.MetadataUpdated(address(this), "https://service.example.com", "Service Name");

        registry.registerAsService("https://service.example.com", "Service Name");
        assertTrue(registry.isService(address(this)), "Service should be registered");
    }

    function test_RegisterAsService_AlreadyRegistered() public {
        registry.registerAsService("http://uri.com", "Name");

        vm.expectRevert("Already registered");
        registry.registerAsService("http://uri2.com", "Name 2");
    }

    function test_RegisterAsOperator() public {
        vm.expectEmit();
        emit SLAYRegistry.OperatorRegistered(address(this));

        vm.expectEmit();
        emit SLAYRegistry.MetadataUpdated(address(this), "https://operator.com", "Operator X");

        registry.registerAsOperator("https://operator.com", "Operator X");
        assertTrue(registry.isOperator(address(this)), "Operator should be registered");
    }

    function test_RegisterAsOperator_AlreadyRegistered() public {
        registry.registerAsOperator("http://operator.com", "Operator X");

        vm.expectRevert("Already registered");
        registry.registerAsOperator("http://operating.com", "Operator X2");
    }

    /**
     * Tests the full registration flow initiated by the service.
     * 1. Service registers the operator (status -> ServiceRegistered).
     * 2. Operator registers the service (status -> Active).
     */
    function test_FullFlow_ServiceInitiatesRegistration() public {
        {
            vm.prank(service);
            registry.registerAsService("service.com", "Service A");
            vm.prank(operator);
            registry.registerAsOperator("operator.com", "Operator X");
        }

        // Step 1: Service registers operator
        vm.prank(service);
        vm.expectEmit();
        emit SLAYRegistry.RegistrationStatusUpdated(
            service, operator, SLAYRegistry.RegistrationStatus.ServiceRegistered
        );
        registry.registerOperatorToService(operator);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered"
        );

        // Step 2: Operator accept and register to the service
        vm.prank(operator);
        vm.expectEmit();
        emit SLAYRegistry.RegistrationStatusUpdated(service, operator, SLAYRegistry.RegistrationStatus.Active);
        registry.registerServiceToOperator(service);
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active"
        );
    }

    /**
     * Tests the full registration flow initiated by the operator.
     * 1. Operator registers the service (status -> OperatorRegistered).
     * 2. Service registers the operator (status -> Active).
     */
    function test_FullFlow_OperatorInitiatesRegistration() public {
        {
            vm.prank(service);
            registry.registerAsService("service.com", "Service A");
            vm.prank(operator);
            registry.registerAsOperator("operator.com", "Operator X");
        }

        // Step 1: Operator registers service
        vm.prank(operator);
        vm.expectEmit();
        emit SLAYRegistry.RegistrationStatusUpdated(
            service, operator, SLAYRegistry.RegistrationStatus.OperatorRegistered
        );
        registry.registerServiceToOperator(service);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.OperatorRegistered),
            "Status should be OperatorRegistered"
        );

        // Step 2: Service accept and register operator
        vm.prank(service);
        vm.expectEmit();
        emit SLAYRegistry.RegistrationStatusUpdated(service, operator, SLAYRegistry.RegistrationStatus.Active);
        registry.registerOperatorToService(operator);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active"
        );
    }

    function test_DeregisterOperatorFromService() public {
        test_FullFlow_ServiceInitiatesRegistration();

        vm.prank(service);
        vm.expectEmit();
        emit SLAYRegistry.RegistrationStatusUpdated(service, operator, SLAYRegistry.RegistrationStatus.Inactive);
        registry.deregisterOperatorFromService(operator);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive after deregistration"
        );
    }

    function test_DeregisterServiceFromOperator() public {
        test_FullFlow_OperatorInitiatesRegistration();

        vm.prank(operator);
        vm.expectEmit();
        emit SLAYRegistry.RegistrationStatusUpdated(service, operator, SLAYRegistry.RegistrationStatus.Inactive);
        registry.deregisterServiceFromOperator(service);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive after deregistration"
        );
    }

    function _advanceBlockBy(uint256 newHeight) internal {
        vm.roll(block.number + newHeight);
        vm.warp(block.timestamp + (12 * newHeight));
    }

    function test_GetRegistrationStatusAt() public {
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
            uint256(registry.getRegistrationStatusAt(service, operator, timeBeforeRegister)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive because no prior history"
        );

        // Inactive at current time
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive because no prior history"
        );

        _advanceBlockBy(1);

        // Service initiates registration
        vm.prank(service);
        registry.registerOperatorToService(operator);
        uint32 timeAfterRegister = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeBeforeRegister)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive prior to registration"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeAfterRegister)),
            uint256(SLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered"
        );
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered at current time"
        );

        _advanceBlockBy(100);

        // Check previous block status
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeBeforeRegister)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should still be Inactive"
        );

        // Operator completes registration
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        uint32 timeAfterActive = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeAfterActive)),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active"
        );
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active at current time"
        );

        // Check all previous checkpoint
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, 0)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive at timestamp 0"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeBeforeRegister)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive before registration"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeAfterRegister)),
            uint256(SLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered after registration"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeAfterActive)),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active after mutual registration"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, uint32(block.timestamp + 1000000000))),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active in the future"
        );
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active at current time"
        );
    }

    function test_Fail_UnregisteredService() public {
        vm.expectRevert(abi.encodeWithSelector(SLAYRegistry.ServiceNotFound.selector, address(this)));
        registry.registerOperatorToService(operator);
    }

    function test_Fail_UnregisteredOperator() public {
        vm.expectRevert(abi.encodeWithSelector(SLAYRegistry.OperatorNotFound.selector, address(this)));
        registry.registerServiceToOperator(service);
    }

    function test_Fail_RegisterOperatorToService_UnregisteredOperator() public {
        registry.registerAsService("https://service.eth", "Service A");

        vm.expectRevert(abi.encodeWithSelector(SLAYRegistry.OperatorNotFound.selector, address(operator)));
        registry.registerOperatorToService(operator);
    }

    function test_Fail_RegisterServiceToOperator_UnregisteredService() public {
        registry.registerAsOperator("https://operator.eth", "Operator X");

        vm.expectRevert(abi.encodeWithSelector(SLAYRegistry.ServiceNotFound.selector, address(service)));
        registry.registerServiceToOperator(service);
    }

    function test_Fail_RegisterOperatorToService_AlreadyActive() public {
        test_FullFlow_ServiceInitiatesRegistration();

        vm.prank(service);
        vm.expectRevert("Already active");
        registry.registerOperatorToService(operator);
    }

    function test_Fail_RegisterServiceToOperator_AlreadyActive() public {
        test_FullFlow_ServiceInitiatesRegistration();

        vm.prank(operator);
        vm.expectRevert("Already active");
        registry.registerServiceToOperator(service);
    }
}
