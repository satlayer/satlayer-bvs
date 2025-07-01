// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {PausableUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";

import {SLAYRegistry, SlashParameter, ServiceOperator} from "../src/SLAYRegistry.sol";
import {ISLAYRegistry} from "../src/interface/ISLAYRegistry.sol";
import {SLAYRegistry, SlashParameter} from "../src/SLAYRegistry.sol";
import {SLAYRouter} from "../src/SLAYRouter.sol";
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
        emit ISLAYRegistry.RegistrationStatusUpdated(
            service, operator, ISLAYRegistry.RegistrationStatus.ServiceRegistered
        );
        registry.registerOperatorToService(operator);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered"
        );

        // Step 2: Operator accept and register to the service
        vm.prank(operator);
        vm.expectEmit();
        emit ISLAYRegistry.RegistrationStatusUpdated(service, operator, ISLAYRegistry.RegistrationStatus.Active);
        registry.registerServiceToOperator(service);
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.Active),
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
        emit ISLAYRegistry.RegistrationStatusUpdated(
            service, operator, ISLAYRegistry.RegistrationStatus.OperatorRegistered
        );
        registry.registerServiceToOperator(service);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.OperatorRegistered),
            "Status should be OperatorRegistered"
        );

        // Step 2: Service accept and register operator
        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRegistry.RegistrationStatusUpdated(service, operator, ISLAYRegistry.RegistrationStatus.Active);
        registry.registerOperatorToService(operator);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.Active),
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
        test_FullFlow_ServiceInitiatesRegistration();

        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRegistry.RegistrationStatusUpdated(service, operator, ISLAYRegistry.RegistrationStatus.Inactive);
        registry.deregisterOperatorFromService(operator);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive after deregistration"
        );
    }

    function test_DeregisterServiceFromOperator() public {
        test_FullFlow_OperatorInitiatesRegistration();

        vm.prank(operator);
        vm.expectEmit();
        emit ISLAYRegistry.RegistrationStatusUpdated(service, operator, ISLAYRegistry.RegistrationStatus.Inactive);
        registry.deregisterServiceFromOperator(service);

        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive after deregistration"
        );
    }

    function test_Deregister_Paused() public {
        test_FullFlow_ServiceInitiatesRegistration();

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
            uint256(ISLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive because no prior history"
        );

        // Inactive at current time
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive because no prior history"
        );

        _advanceBlockBy(1);

        // Service initiates registration
        vm.prank(service);
        registry.registerOperatorToService(operator);
        uint32 timeAfterRegister = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeBeforeRegister)),
            uint256(ISLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive prior to registration"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeAfterRegister)),
            uint256(ISLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered"
        );
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered at current time"
        );

        _advanceBlockBy(100);

        // Check previous block status
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeBeforeRegister)),
            uint256(ISLAYRegistry.RegistrationStatus.Inactive),
            "Status should still be Inactive"
        );

        // Operator completes registration
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        uint32 timeAfterActive = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeAfterActive)),
            uint256(ISLAYRegistry.RegistrationStatus.Active),
            "Status should be Active"
        );
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.Active),
            "Status should be Active at current time"
        );

        // Check all previous checkpoint
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, 0)),
            uint256(ISLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive at timestamp 0"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeBeforeRegister)),
            uint256(ISLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive before registration"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeAfterRegister)),
            uint256(ISLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered after registration"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, timeAfterActive)),
            uint256(ISLAYRegistry.RegistrationStatus.Active),
            "Status should be Active after mutual registration"
        );
        assertEq(
            uint256(registry.getRegistrationStatusAt(service, operator, uint32(block.timestamp + 1000000000))),
            uint256(ISLAYRegistry.RegistrationStatus.Active),
            "Status should be Active in the future"
        );
        assertEq(
            uint256(registry.getRegistrationStatus(service, operator)),
            uint256(ISLAYRegistry.RegistrationStatus.Active),
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

    function test_EnableSlashing() public {
        vm.prank(service);
        registry.registerAsService("service.com", "Service A");

        address destination = makeAddr("slashDestination");
        uint32 maxBips = 5000;
        uint32 resolutionWindow = 3600;

        vm.prank(service);
        vm.expectEmit();
        emit SLAYRegistry.SlashParameterUpdated(service, destination, maxBips, resolutionWindow);
        registry.enableSlashing(
            SlashParameter.Object({destination: destination, maxMilliBips: maxBips, resolutionWindow: resolutionWindow})
        );

        SlashParameter.Object memory param = registry.getSlashParameter(service);

        assertEq(param.destination, destination, "Slashing destination should match");
        assertEq(param.maxMilliBips, maxBips, "Slashing maxBips should match");
        assertEq(param.resolutionWindow, resolutionWindow, "Slashing resolutionWindow should match");
    }

    function test_EnableSlashing_MaxBipsEdgeCases() public {
        vm.prank(service);
        registry.registerAsService("service.com", "Service A");

        address destination = makeAddr("slashDestination");
        uint32 resolutionWindow = 3600;

        vm.prank(service);
        registry.enableSlashing(
            SlashParameter.Object({destination: destination, maxMilliBips: 10000000, resolutionWindow: resolutionWindow})
        );

        SlashParameter.Object memory param = registry.getSlashParameter(service);
        assertEq(param.maxMilliBips, 10000000, "MaxBips should be 10000000");

        // Test maxBips at 10000001 (revert)
        vm.prank(service);
        vm.expectRevert("Maximum Milli-Bips cannot be more than 10_000_000 (100%)");
        registry.enableSlashing(
            SlashParameter.Object({destination: destination, maxMilliBips: 10000001, resolutionWindow: resolutionWindow})
        );
    }

    function test_EnableSlashing_NotService() public {
        address nonService = makeAddr("nonService");
        vm.prank(nonService);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistry.ServiceNotFound.selector, nonService));
        registry.enableSlashing(
            SlashParameter.Object({destination: makeAddr("dest"), maxMilliBips: 100, resolutionWindow: 1000})
        );
    }

    function test_GetSlashParameterAt() public {
        vm.prank(service);
        registry.registerAsService("service.com", "Service A");

        address destination1 = makeAddr("slashDestination1");
        uint32 maxBips1 = 100;
        uint32 resolutionWindow1 = 1000;

        vm.prank(service);
        registry.enableSlashing(
            SlashParameter.Object({
                destination: destination1,
                maxMilliBips: maxBips1,
                resolutionWindow: resolutionWindow1
            })
        );

        uint32 time1 = uint32(block.timestamp);

        _advanceBlockBy(10); // Advance time

        address destination2 = makeAddr("slashDestination2");
        uint32 maxBips2 = 200;
        uint32 resolutionWindow2 = 2000;

        vm.prank(service);
        registry.enableSlashing(
            SlashParameter.Object({
                destination: destination2,
                maxMilliBips: maxBips2,
                resolutionWindow: resolutionWindow2
            })
        );
        uint32 time2 = uint32(block.timestamp);

        // Check at time1
        SlashParameter.Object memory param1 = registry.getSlashParameterAt(service, time1);
        assertEq(param1.destination, destination1, "Slashing destination at time1 should match");
        assertEq(param1.maxMilliBips, maxBips1, "Slashing maxBips at time1 should match");
        assertEq(param1.resolutionWindow, resolutionWindow1, "Slashing resolutionWindow at time1 should match");

        // Check at time2
        SlashParameter.Object memory param2 = registry.getSlashParameterAt(service, time2);
        assertEq(param2.destination, destination2, "Slashing destination at time2 should match");
        assertEq(param2.maxMilliBips, maxBips2, "Slashing maxBips at time2 should match");
        assertEq(param2.resolutionWindow, resolutionWindow2, "Slashing resolutionWindow at time2 should match");

        // Check a time before any update (should return default/zero values)
        SlashParameter.Object memory param3 = registry.getSlashParameterAt(service, 0);
        assertEq(param3.destination, address(0), "Slashing destination at time 0 should be zero address");
        assertEq(param3.maxMilliBips, 0, "Slashing maxBips at time 0 should be 0");
        assertEq(param3.resolutionWindow, 0, "Slashing resolutionWindow at time 0 should be 0");
    }

    function test_SlashingOptIn() public {
        test_FullFlow_ServiceInitiatesRegistration(); // Ensures service and operator are active

        address destination1 = makeAddr("slashDestination1");
        uint32 maxBips1 = 100;
        uint32 resolutionWindow1 = 1000;

        vm.prank(service);
        registry.enableSlashing(
            SlashParameter.Object({
                destination: destination1,
                maxMilliBips: maxBips1,
                resolutionWindow: resolutionWindow1
            })
        );

        vm.prank(operator);
        vm.expectEmit();
        emit SLAYRegistry.SlashOptIn(service, operator);
        registry.slashOptIn(service);

        assertTrue(registry.getSlashOptIns(service, operator), "Operator should have opted in for slashing");
    }

    function test_SlashingOptIn_AlreadyOptedIn() public {
        test_SlashingOptIn(); // Opt-in once

        vm.prank(operator);
        // Expect no revert, as re-opting-in should simply update the checkpoint to the current time.
        // The current implementation allows re-opting-in without a specific revert.
        vm.expectRevert("Operator already opted in slashing for this service");
        registry.slashOptIn(service);
    }

    function test_SlashOptIn_NotService() public {
        vm.prank(operator);
        registry.registerAsOperator("op.com", "Op");

        address nonService = makeAddr("nonService");

        vm.expectRevert("RegistrationStatus not Active");
        registry.slashOptIn(nonService);
    }

    function test_SlashingOptIn_NotOperator() public {
        vm.prank(service);
        registry.registerAsService("service.com", "Service A");

        address nonOperator = makeAddr("nonOperator");
        vm.prank(nonOperator);
        vm.expectRevert("RegistrationStatus not Active");
        registry.slashOptIn(service);
    }

    function test_SlashingOptIn_RegistrationNotActive() public {
        // Service and operator are registered but not actively paired
        vm.prank(service);
        registry.registerAsService("service.com", "Service A");
        vm.prank(operator);
        registry.registerAsOperator("operator.com", "Operator X");

        // Attempt opt-in when status is Inactive
        vm.prank(operator);
        vm.expectRevert("RegistrationStatus not Active");
        registry.slashOptIn(service);

        // Service initiates, status is ServiceRegistered
        vm.prank(service);
        registry.registerOperatorToService(operator);

        vm.prank(operator);
        vm.expectRevert("RegistrationStatus not Active");
        registry.slashOptIn(service);
    }

    function test_GetSlashingOptInsAt() public {
        test_FullFlow_ServiceInitiatesRegistration(); // Ensures active registration

        _advanceBlockBy(2);

        vm.prank(service);
        SlashParameter.Object memory slashParams =
            SlashParameter.Object({destination: address(service), maxMilliBips: 10000000, resolutionWindow: 10000000});
        registry.enableSlashing(slashParams);

        // Initial state before opt-in
        assertTrue(!registry.getSlashOptInsAt(service, operator, 0), "Should not be opted in at timestamp 0");
        assertTrue(
            !registry.getSlashOptInsAt(service, operator, uint32(block.timestamp)), "Should not be opted in initially"
        );

        vm.prank(operator);
        registry.slashOptIn(service);
        uint32 timeAfterOptIn = uint32(block.timestamp);

        _advanceBlockBy(5); // Advance time

        // Check after opt-in
        assertTrue(
            registry.getSlashOptInsAt(service, operator, timeAfterOptIn), "Should be opted in after opt-in event"
        );
        assertTrue(
            registry.getSlashOptInsAt(service, operator, uint32(block.timestamp)),
            "Should be opted in at current timestamp"
        );

        assertTrue(
            registry.getSlashOptInsAt(service, operator, uint32(block.timestamp + 1000000)),
            "Should be opted in at a future timestamp"
        );
    }

    function test_SlashOptOut_Implicit() public {
        test_FullFlow_ServiceInitiatesRegistration(); // Ensures active registration

        _advanceBlockBy(2);

        vm.prank(service);
        SlashParameter.Object memory slashParams =
            SlashParameter.Object({destination: address(service), maxMilliBips: 10000000, resolutionWindow: 10000000});
        registry.enableSlashing(slashParams);

        // Initial state before opt-in
        assertTrue(!registry.getSlashOptInsAt(service, operator, 0), "Should not be opted in at timestamp 0");
        assertTrue(
            !registry.getSlashOptInsAt(service, operator, uint32(block.timestamp)), "Should not be opted in initially"
        );

        vm.prank(operator);
        registry.slashOptIn(service);
        uint32 timeAfterOptIn = uint32(block.timestamp);

        _advanceBlockBy(5); // Advance time

        // Check after opt-in
        assertTrue(
            registry.getSlashOptInsAt(service, operator, timeAfterOptIn), "Should be opted in after opt-in event"
        );

        _advanceBlockBy(10);

        vm.prank(service);
        SlashParameter.Object memory slashParams2 =
            SlashParameter.Object({destination: address(0), maxMilliBips: 200, resolutionWindow: 200});
        registry.enableSlashing(slashParams2);

        _advanceBlockBy(10);

        // Check after new slashing parameter. Operator should be opted out implicitly.
        assertFalse(
            registry.getSlashOptIns(service, operator),
            "Should be opted out from slash for new slashing parameters implicitly"
        );

        // historical check still works fine.
        assertTrue(
            registry.getSlashOptInsAt(service, operator, timeAfterOptIn), "Should be opted in at the provided timestamp"
        );
    }
}
