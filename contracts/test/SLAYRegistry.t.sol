// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import {Test, console} from "forge-std/Test.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import {SLAYRegistry} from "../src/SLAYRegistry.sol";
import {SLAYRouter} from "../src/SLAYRouter.sol";
import {EmptyImpl} from "../src/EmptyImpl.sol";

contract SLAYRegistryTest is Test {
    SLAYRegistry public registry;
    SLAYRouter public router;

    address public owner;
    address public service;
    address public operator;
    address public anotherUser;

    /**
     * @dev Sets up the test environment.
     * This setup handles the deployment of UUPS upgradeable proxies with cyclic dependencies
     * between SLAYRegistry and SLAYRouter.
     * 1. Deploy an EmptyImpl contract to serve as the initial implementation for proxies.
     * 2. Deploy ERC1967Proxy for both Registry and Router, pointing to EmptyImpl.
     * 3. Initialize the EmptyImpl proxies, setting the owner.
     * 4. Deploy the actual SLAYRegistry and SLAYRouter logic contracts, providing the
     * respective proxy addresses to their constructors to resolve the cyclic dependency.
     * 5. Upgrade the proxies from EmptyImpl to the actual logic contracts.
     * 6. Initialize the final Registry and Router contracts through their proxies.
     */
    function setUp() public {
        /**
         * --- Initialize addresses ---
         */
        owner = makeAddr("owner");
        service = makeAddr("service");
        operator = makeAddr("operator");
        anotherUser = makeAddr("anotherUser");

        vm.prank(owner);

        /**
         * Deploy Empty Implementation
         */
        EmptyImpl emptyImpl = new EmptyImpl();

        /**
         * Deploy and Initialize Proxies with EmptyImpl
         */
        bytes memory emptyImplInitData = abi.encodeWithSelector(EmptyImpl.initialize.selector, owner);

        ERC1967Proxy registryProxy = new ERC1967Proxy(address(emptyImpl), emptyImplInitData);
        registry = SLAYRegistry(address(registryProxy));

        ERC1967Proxy routerProxy = new ERC1967Proxy(address(emptyImpl), emptyImplInitData);
        router = SLAYRouter(address(routerProxy));

        /**
         * Deploy Logic Contracts with cyclic dependency
         */
        SLAYRouter routerLogic = new SLAYRouter(registry);
        SLAYRegistry registryLogic = new SLAYRegistry(router);

        /**
         * Upgrade Proxies to Logic Contracts
         */
        vm.prank(owner);
        registry.upgradeToAndCall(address(registryLogic), "");
        vm.prank(owner);
        router.upgradeToAndCall(address(routerLogic), "");

        /**
         * Initialize Logic Contracts
         */
        vm.prank(owner);
        registry.initialize();
        vm.prank(owner);
        router.initialize();
    }

    /**
     * --- Test Registration Functions ---
     */
    function test_RegisterAsService() public {
        vm.prank(service);
        vm.expectEmit(true, true, true, true);
        emit SLAYRegistry.ServiceRegistered(service, "service_uri", "Service A");
        registry.registerAsService("service_uri", "Service A");
        assertTrue(registry.services(service), "Service should be registered");
    }

    function test_Fail_RegisterAsService_AlreadyRegistered() public {
        vm.prank(service);
        registry.registerAsService("service_uri", "Service A");

        vm.prank(service);
        vm.expectRevert("Service has been registered");
        registry.registerAsService("service_uri_2", "Service A2");
    }

    function test_RegisterAsOperator() public {
        vm.prank(operator);
        vm.expectEmit(true, true, true, true);
        emit SLAYRegistry.OperatorRegistered(operator, "operator_uri", "Operator X");
        registry.registerAsOperator("operator_uri", "Operator X");
        assertTrue(registry.operators(operator), "Operator should be registered");
    }

    function test_Fail_RegisterAsOperator_AlreadyRegistered() public {
        vm.prank(operator);
        registry.registerAsOperator("operator_uri", "Operator X");

        vm.prank(operator);
        vm.expectRevert("Operator has been registerd");
        registry.registerAsOperator("operator_uri_2", "Operator X2");
    }

    /**
     * --- Test Registration Flow ---
     */
    function _registerServiceAndOperator() internal {
        vm.prank(service);
        registry.registerAsService("service_uri", "Service A");
        vm.prank(operator);
        registry.registerAsOperator("operator_uri", "Operator X");
    }

    /**
     * Tests the full registration flow initiated by the service.
     * 1. Service registers the operator (status -> ServiceRegistered).
     * 2. Operator registers the service (status -> Active).
     */
    function test_FullFlow_ServiceInitiatesRegistration() public {
        _registerServiceAndOperator();
        bytes32 key = registry._getKey(service, operator);

        /**
         * --- Step 1: Service registers operator ---
         */
        vm.prank(service);
        registry.registerOperatorToService(operator);

        assertEq(
            uint256(registry.getLatestRegistrationStatus(key)),
            uint256(SLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered"
        );

        /**
         * --- Step 2: Operator accept and register to the service ---
         */
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        assertEq(
            uint256(registry.getLatestRegistrationStatus(key)),
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
        _registerServiceAndOperator();
        bytes32 key = registry._getKey(service, operator);

        /**
         * --- Step 1: Operator registers service ---
         */
        vm.prank(operator);
        vm.expectEmit(true, true, true, true);
        emit SLAYRegistry.RegistrationStatusUpdated(
            service, operator, SLAYRegistry.RegistrationStatus.OperatorRegistered
        );
        registry.registerServiceToOperator(service);

        assertEq(
            uint256(registry.getLatestRegistrationStatus(key)),
            uint256(SLAYRegistry.RegistrationStatus.OperatorRegistered),
            "Status should be OperatorRegistered"
        );

        /**
         * -- Step 2: Service accept and register operator
         */
        vm.prank(service);
        registry.registerOperatorToService(operator);

        assertEq(
            uint256(registry.getLatestRegistrationStatus(key)),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active"
        );
    }

    /**
     * --- Test Deregistration Functions ---
     */
    function test_DeregisterOperatorFromService() public {
        test_FullFlow_ServiceInitiatesRegistration();
        bytes32 key = registry._getKey(service, operator);

        vm.prank(service);
        registry.deregisterOperatorFromService(operator);

        assertEq(
            uint256(registry.getLatestRegistrationStatus(key)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive after deregistration"
        );
    }

    function test_DeregisterServiceFromOperator() public {
        // First, complete registration
        test_FullFlow_OperatorInitiatesRegistration();
        bytes32 key = registry._getKey(service, operator);

        // Now, deregister
        vm.prank(operator);
        vm.expectEmit(true, true, true, true);
        emit SLAYRegistry.RegistrationStatusUpdated(service, operator, SLAYRegistry.RegistrationStatus.Inactive);
        registry.deregisterServiceFromOperator(service);

        assertEq(
            uint256(registry.getLatestRegistrationStatus(key)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive after deregistration"
        );
    }

    // --- Test Checkpoints and Status Lookups ---

    function advanceBlockBy(uint256 newHeight) public {
        uint256 currentBlockTime = block.timestamp;
        vm.roll(block.number + newHeight);
        vm.warp(currentBlockTime + (12 * newHeight));
    }

    function test_GetRegistrationStatusAt() public {
        _registerServiceAndOperator();

        advanceBlockBy(1);
        bytes32 key = registry._getKey(service, operator);

        // Initial state is Inactive
        uint32 timeBeforeRegister = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRegistrationStatusAt(key, timeBeforeRegister)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive because no prior history"
        );

        advanceBlockBy(1);

        // Service initiates registration
        vm.prank(service);
        registry.registerOperatorToService(operator);
        uint32 timeAfterRegister = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRegistrationStatusAt(key, timeAfterRegister)),
            uint256(SLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered"
        );

        advanceBlockBy(100);

        // Check previous block status
        assertEq(
            uint256(registry.getRegistrationStatusAt(key, timeBeforeRegister)),
            uint256(SLAYRegistry.RegistrationStatus.Inactive),
            "Status should be Inactive"
        );

        // Operator completes registration
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        uint32 timeAfterActive = uint32(block.timestamp);
        assertEq(
            uint256(registry.getRegistrationStatusAt(key, timeAfterActive)),
            uint256(SLAYRegistry.RegistrationStatus.Active),
            "Status should be Active"
        );
        // Check previous block status
        assertEq(
            uint256(registry.getRegistrationStatusAt(key, timeAfterRegister)),
            uint256(SLAYRegistry.RegistrationStatus.ServiceRegistered),
            "Status should be ServiceRegistered"
        );
    }

    // --- Test Revert Conditions ---

    function test_Fail_RegisterOperatorToService_UnregisteredOperator() public {
        vm.prank(service);
        registry.registerAsService("service_uri", "Service A");

        vm.prank(service);
        vm.expectRevert("Operator not found");
        registry.registerOperatorToService(operator);
    }

    function test_Fail_RegisterServiceToOperator_UnregisteredService() public {
        vm.prank(operator);
        registry.registerAsOperator("operator_uri", "Operator X");

        vm.prank(operator);
        vm.expectRevert("Service not registered");
        registry.registerServiceToOperator(service);
    }

    function test_Fail_RegisterOperatorToService_AlreadyActive() public {
        test_FullFlow_ServiceInitiatesRegistration();

        vm.prank(service);
        vm.expectRevert("Registration is already active");
        registry.registerOperatorToService(operator);
    }

    function test_Fail_RegisterServiceToOperator_AlreadyActive() public {
        test_FullFlow_ServiceInitiatesRegistration();

        vm.prank(operator);
        vm.expectRevert("Registration between operator and service is already active");
        registry.registerServiceToOperator(service);
    }
}
