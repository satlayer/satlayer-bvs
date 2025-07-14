// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "./MockERC20.sol";
import "../src/SLAYRouterV2.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "./TestSuiteV2.sol";
import {ISLAYRouterV2} from "../src/interface/ISLAYRouterV2.sol";
import {ISLAYRouterSlashingV2} from "../src/interface/ISLAYRouterSlashingV2.sol";
import {ISLAYRegistryV2} from "../src/interface/ISLAYRegistryV2.sol";

contract SLAYRouterV2Test is Test, TestSuiteV2 {
    function test_defaults() public view {
        assertEq(router.owner(), owner);
        assertEq(router.paused(), false);
    }

    function test_paused() public {
        vm.prank(owner);
        router.pause();

        assertTrue(router.paused());
    }

    function test_pausedOnlyOwnerError() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.pause();
    }

    function test_unpausedOnlyOwnerError() public {
        vm.prank(owner);
        router.pause();

        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.unpause();
    }

    function test_Whitelisted() public {
        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        assertFalse(router.isVaultWhitelisted(vault));

        vm.prank(owner);
        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(operator, vault, true);
        router.setVaultWhitelist(vault, true);

        assertTrue(router.isVaultWhitelisted(vault));

        vm.prank(owner);
        vm.expectEmit();
        emit ISLAYRouterV2.VaultWhitelisted(operator, vault, false);
        router.setVaultWhitelist(vault, false);

        assertFalse(router.isVaultWhitelisted(vault));
    }

    function test_Whitelisted_NotVault() public {
        address vault = vm.randomAddress();
        assertFalse(router.isVaultWhitelisted(vault));

        vm.prank(owner);
        vm.expectRevert();
        router.setVaultWhitelist(vault, true);

        assertFalse(router.isVaultWhitelisted(vault));
    }

    function test_Whitelisted_AlreadyWhitelisted() public {
        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        assertFalse(router.isVaultWhitelisted(vault));

        vm.startPrank(owner);
        router.setVaultWhitelist(vault, true);
        assertTrue(router.isVaultWhitelisted(vault));

        vm.expectRevert("Vault already in desired state");
        router.setVaultWhitelist(vault, true);
        assertTrue(router.isVaultWhitelisted(vault));

        router.setVaultWhitelist(vault, false);
        assertFalse(router.isVaultWhitelisted(vault));
        vm.stopPrank();
    }

    function test_Whitelisted_ExceedsMaxVaultsPerOperator() public {
        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        for (uint256 i = 0; i < 10; i++) {
            vm.prank(operator);
            address vaultI = address(vaultFactory.create(underlying));
            vm.prank(owner);
            router.setVaultWhitelist(vaultI, true);
            assertTrue(router.isVaultWhitelisted(vaultI));
        }

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));

        vm.prank(owner);

        vm.expectRevert("Exceeds max vaults per operator");
        router.setVaultWhitelist(vault, true);

        assertFalse(router.isVaultWhitelisted(vault));
    }

    function test_Whitelisted_NewVaultsCanBeAddedAfterRemoval() public {
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        address[] memory vaults = new address[](10);

        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        for (uint256 i = 0; i < 10; i++) {
            vm.prank(operator);
            vaults[i] = address(vaultFactory.create(underlying));

            vm.prank(owner);
            router.setVaultWhitelist(vaults[i], true);
        }

        vm.prank(operator);
        address newVault = address(vaultFactory.create(underlying));
        assertFalse(router.isVaultWhitelisted(newVault));

        vm.prank(owner);
        router.setVaultWhitelist(vaults[0], false);
        assertFalse(router.isVaultWhitelisted(vaults[0]));

        vm.prank(owner);
        router.setVaultWhitelist(newVault, true);
        assertTrue(router.isVaultWhitelisted(newVault));
    }

    function test_OnlyOwnerCanSetWhitelist() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.setVaultWhitelist(address(0), true);
    }

    function test_setMaxVaultsPerOperator() public {
        vm.prank(owner);
        router.setMaxVaultsPerOperator(20);
        assertEq(router.getMaxVaultsPerOperator(), 20);

        address operator = makeAddr("Operator Y");
        vm.prank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");

        MockERC20 underlying = new MockERC20("Token", "TKN", 18);

        for (uint256 i = 0; i < 20; i++) {
            vm.prank(operator);
            address vaultI = address(vaultFactory.create(underlying));

            vm.prank(owner);
            router.setVaultWhitelist(vaultI, true);
        }

        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));

        vm.prank(owner);
        vm.expectRevert("Exceeds max vaults per operator");
        router.setVaultWhitelist(vault, true);
        assertFalse(router.isVaultWhitelisted(vault));
    }

    function test_setMaxVaultsPerOperator_OnlyOwner() public {
        vm.expectRevert(abi.encodeWithSelector(OwnableUpgradeable.OwnableUnauthorizedAccount.selector, address(this)));
        router.setMaxVaultsPerOperator(20);
    }

    function test_setMaxVaultsPerOperator_MustBeGreaterThanCurrent() public {
        vm.prank(owner);
        vm.expectRevert("Must be greater than current");
        router.setMaxVaultsPerOperator(0);
    }

    function test_setMaxVaultsPerOperator_InitialValue() public {
        assertEq(router.getMaxVaultsPerOperator(), 10);

        vm.prank(owner);
        router.setMaxVaultsPerOperator(15);
        assertEq(router.getMaxVaultsPerOperator(), 15);
    }

    function test_slashRequest_ideal() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        vm.prank(operator);
        registry.registerAsOperator("operator.com", "Operator X");

        vm.startPrank(service);
        registry.registerAsService("service.com", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: vm.randomAddress(), maxMbips: 100_000, resolutionWindow: 3600})
        );
        registry.registerOperatorToService(operator);
        vm.stopPrank();

        vm.prank(operator);
        registry.registerServiceToOperator(service);

        _advanceBlockBy(10);

        vm.prank(operator);
        registry.approveSlashingFor(service);

        uint32 newDelay = 8 days;
        vm.prank(operator);
        registry.setWithdrawalDelay(newDelay);

        _advanceBlockBy(10);

        uint32 timeAtWhichOffenseOccurs = uint32(block.timestamp);

        _advanceBlockBy(10);

        vm.prank(service);
        ISLAYRouterSlashingV2.Payload memory payload = ISLAYRouterSlashingV2.Payload({
            operator: operator,
            mbips: 100,
            timestamp: timeAtWhichOffenseOccurs,
            reason: "Missing Blocks"
        });
        router.requestSlashing(payload);

        ISLAYRouterSlashingV2.Request memory info = router.getPendingSlashingRequest(service, operator);

        assertEq(info.operator, operator);
        assertEq(info.timestamp, timeAtWhichOffenseOccurs);
        assertEq(info.mbips, 100);
        assertTrue(info.status == ISLAYRouterSlashingV2.Status.Pending);
        assertEq(info.requestResolution, uint32(block.timestamp) + 3600); // now + resolution window
        assertEq(info.requestExpiry, uint32(block.timestamp) + 3600 + 7 days);
    }

    function test_lockSlashing() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");
        address[5] memory vaults;

        // create 5 vaults for operator and fund them
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;

        // register operator
        vm.prank(operator);
        registry.registerAsOperator("operator", "Operator X");

        for (uint256 i = 0; i < 5; i++) {
            vm.prank(operator);
            address vault = address(vaultFactory.create(underlying));

            vm.prank(owner);
            router.setVaultWhitelist(vault, true);

            underlying.mint(vault, (i + 1) * 1_000_000 * underlyingMinorUnit); // mint 1m, 2m, ..., 5m to each vault
            vaults[i] = vault;
        }

        // register service and enable slashing
        vm.startPrank(service);
        registry.registerAsService("service", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: service, maxMbips: 1_000_000, resolutionWindow: 3600})
        );
        vm.stopPrank();

        // register service to operator and vice versa
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        vm.prank(service);
        registry.registerOperatorToService(operator);

        // TODO: remove after fix SL-620
        vm.prank(operator);
        registry.approveSlashingFor(service);

        _advanceBlockBy(1000);

        // Service initiates slashing request
        vm.prank(service);
        ISLAYRouterSlashingV2.Payload memory payload = ISLAYRouterSlashingV2.Payload({
            operator: operator,
            mbips: 1_000_000, // 10%
            timestamp: uint32(block.timestamp) - 100,
            reason: "Missing Blocks"
        });
        bytes32 slashId = router.requestSlashing(payload);

        ISLAYRouterSlashingV2.Request memory pendingRequest = router.getPendingSlashingRequest(service, operator);
        assertTrue(pendingRequest.status == ISLAYRouterSlashingV2.Status.Pending);

        // fast forward to after resolution window
        _advanceBlockBy(360);

        // Service initiates lock slashing
        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRouterSlashingV2.SlashingLocked(service, operator, slashId);
        router.lockSlashing(slashId);

        // assert status of request is Locked
        ISLAYRouterSlashingV2.Request memory slashRequestAfterLock = router.getPendingSlashingRequest(service, operator);
        assertTrue(slashRequestAfterLock.status == ISLAYRouterSlashingV2.Status.Locked);

        // assert that the slashed assets are moved to the router
        uint256 routerBalance = MockERC20(underlying).balanceOf(address(router));
        assertEq(routerBalance, 1_500_000 * underlyingMinorUnit); // 10% of 5 vaults

        // assert that the vaults' balance are reduced by 10%
        for (uint256 i = 0; i < 5; i++) {
            address vault = vaults[i];
            uint256 vaultBalance = MockERC20(underlying).balanceOf(vault);
            uint256 balanceBeforeSlashing = (i + 1) * 1_000_000 * underlyingMinorUnit;
            assertEq(vaultBalance, balanceBeforeSlashing * 9 / 10); // (10% slashed)
        }

        // assert that the internal state _lockedAssets are updated
        ISLAYRouterSlashingV2.LockedAssets[] memory lockedAssets = router.getLockedAssets(slashId);
        assertEq(lockedAssets.length, 5); // 5 vaults slashed
        for (uint256 i = 0; i < 5; i++) {
            assertEq(lockedAssets[i].amount, (i + 1) * 100_000 * underlyingMinorUnit); // 10% of each vault
            assertEq(lockedAssets[i].vault, vaults[i]);
        }
    }

    function test_revert_lockSlashing() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        // register operator
        vm.prank(operator);
        registry.registerAsOperator("operator", "Operator X");

        // create a vault for operator and fund it
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;
        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        vm.prank(owner);
        router.setVaultWhitelist(vault, true);
        underlying.mint(vault, 1_000_000 * underlyingMinorUnit); // mint 1m to the vault

        // register service and enable slashing
        vm.startPrank(service);
        registry.registerAsService("service", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: service, maxMbips: 1_000_000, resolutionWindow: 3600})
        );
        vm.stopPrank();

        // register service to operator and vice versa
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        vm.prank(service);
        registry.registerOperatorToService(operator);

        // TODO: remove after fix SL-620
        vm.prank(operator);
        registry.approveSlashingFor(service);

        _advanceBlockBy(1000);

        // Service initiates slashing request
        vm.prank(service);
        ISLAYRouterSlashingV2.Payload memory payload = ISLAYRouterSlashingV2.Payload({
            operator: operator,
            mbips: 1_000_000, // 10%
            timestamp: uint32(block.timestamp) - 100,
            reason: "Missing Blocks"
        });
        bytes32 slashId = router.requestSlashing(payload);

        // get the pending slashing request
        ISLAYRouterSlashingV2.Request memory pendingRequest = router.getPendingSlashingRequest(service, operator);
        assertTrue(pendingRequest.status == ISLAYRouterSlashingV2.Status.Pending);

        // revert when non-service tries to lock slashing
        vm.prank(operator);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistryV2.ServiceNotFound.selector, operator));
        router.lockSlashing(slashId);

        // revert when service tries to lock slashing but is not the one who initiated the request
        address anotherService = makeAddr("Another Service");
        vm.startPrank(anotherService);
        registry.registerAsService("another-service", "Another Service");
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.Unauthorized.selector));
        router.lockSlashing(slashId);
        vm.stopPrank();

        // revert when slashing request has not passed resolution window
        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.SlashingResolutionNotReached.selector));
        router.lockSlashing(slashId);

        // fast forward to after expiry
        _advanceBlockBySeconds(7 days + 3601);

        // revert when slashing request has expired
        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.SlashingRequestExpired.selector));
        router.lockSlashing(slashId);

        // move chain back before expiry
        vm.roll(block.number - 10);
        vm.warp(block.timestamp - 120);

        // service should successfully lock slashing now
        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRouterSlashingV2.SlashingLocked(service, operator, slashId);
        router.lockSlashing(slashId);

        // assert status of request is Locked
        ISLAYRouterSlashingV2.Request memory slashRequestAfterLock = router.getPendingSlashingRequest(service, operator);
        assertTrue(slashRequestAfterLock.status == ISLAYRouterSlashingV2.Status.Locked);

        // assert router balance is increased
        uint256 routerBalance = MockERC20(underlying).balanceOf(address(router));
        assertEq(routerBalance, 100_000 * underlyingMinorUnit); // 10% of 1m

        // revert when service tries to lock slashing again
        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.InvalidStatus.selector));
        router.lockSlashing(slashId);

        // assert that the router balance is still the same
        uint256 routerBalanceAfterSecondLock = MockERC20(underlying).balanceOf(address(router));
        assertEq(routerBalanceAfterSecondLock, routerBalance);
    }

    function test_finalizeSlashing() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");
        address destination = makeAddr("Destination");

        // register operator
        vm.prank(operator);
        registry.registerAsOperator("operator", "Operator X");

        // create a vault for operator and fund it
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;
        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        vm.prank(owner);
        router.setVaultWhitelist(vault, true);
        underlying.mint(vault, 1_000_000 * underlyingMinorUnit); // mint 1m to the vault

        // register service and enable slashing
        vm.startPrank(service);
        registry.registerAsService("service", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: destination, maxMbips: 1_000_000, resolutionWindow: 3600})
        );
        vm.stopPrank();

        // register service to operator and vice versa
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        vm.prank(service);
        registry.registerOperatorToService(operator);

        // enable slashing for operator
        vm.prank(operator);
        registry.approveSlashingFor(service);

        // advance time to allow slashing
        _advanceBlockBy(100);

        // Service initiates slashing request
        ISLAYRouterSlashingV2.Payload memory requestPayload = ISLAYRouterSlashingV2.Payload({
            mbips: 1_000_000, // 10%
            timestamp: uint32(block.timestamp) - 100,
            operator: operator,
            reason: "Missing Blocks"
        });
        vm.prank(service);
        bytes32 slashId = router.requestSlashing(requestPayload);

        // fast forward to after resolution window
        _advanceBlockBySeconds(3600);

        // Service locks slashing
        vm.prank(service);
        router.lockSlashing(slashId);

        // Guardrail votes on the slashing request
        address guardrail = makeAddr("Guardrail");
        vm.prank(owner);
        router.setGuardrail(guardrail);
        vm.prank(guardrail);
        vm.expectEmit();
        emit ISLAYRouterSlashingV2.GuardrailApproval(slashId, true);
        router.guardrailApprove(slashId, true);

        // Service finalizes slashing
        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRouterSlashingV2.SlashingFinalized(service, operator, slashId, destination);
        router.finalizeSlashing(slashId);

        // assert that the slashing request is removed from the service/operator mapping
        ISLAYRouterSlashingV2.Request memory removedRequest = router.getPendingSlashingRequest(service, operator);
        assertEq(removedRequest.operator, address(0));

        // assert status of request is Finalized
        ISLAYRouterSlashingV2.Request memory slashRequestAfterFinalize = router.getSlashingRequest(slashId);
        assertTrue(slashRequestAfterFinalize.status == ISLAYRouterSlashingV2.Status.Finalized);

        // assert that the slashed assets are moved to the destination
        uint256 destinationBalance = MockERC20(underlying).balanceOf(destination);
        assertEq(destinationBalance, 100_000 * underlyingMinorUnit); // 10% of 1m

        // assert that the router balance is reduced to 0
        uint256 routerBalance = MockERC20(underlying).balanceOf(address(router));
        assertEq(routerBalance, 0);
    }

    function test_Revert_finalizeSlashing_guardrail_reject() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        // register operator
        vm.prank(operator);
        registry.registerAsOperator("operator", "Operator X");

        // create a vault for operator and fund it
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;
        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        vm.prank(owner);
        router.setVaultWhitelist(vault, true);
        underlying.mint(vault, 1_000_000 * underlyingMinorUnit); // mint 1m to the vault

        // register service and enable slashing
        vm.startPrank(service);
        registry.registerAsService("service", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: service, maxMbips: 1_000_000, resolutionWindow: 3600})
        );
        vm.stopPrank();

        // register service to operator and vice versa
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        vm.prank(service);
        registry.registerOperatorToService(operator);

        // enable slashing for operator
        vm.prank(operator);
        registry.approveSlashingFor(service);

        // advance time to allow slashing
        _advanceBlockBy(100);

        // Service initiates slashing request
        ISLAYRouterSlashingV2.Payload memory requestPayload = ISLAYRouterSlashingV2.Payload({
            mbips: 1_000_000, // 10%
            timestamp: uint32(block.timestamp) - 100,
            operator: operator,
            reason: "Missing Blocks"
        });
        vm.prank(service);
        bytes32 slashId = router.requestSlashing(requestPayload);

        // fast forward to after resolution window
        _advanceBlockBySeconds(3600);
        // Service locks slashing
        vm.prank(service);
        router.lockSlashing(slashId);

        // Guardrail rejects the slashing request
        address guardrail = makeAddr("Guardrail");
        vm.prank(owner);
        router.setGuardrail(guardrail);
        vm.prank(guardrail);
        vm.expectEmit();
        emit ISLAYRouterSlashingV2.GuardrailApproval(slashId, false);
        router.guardrailApprove(slashId, false);

        // Revert when service tries to finalize slashing due to guardrail rejection
        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.GuardrailHaveNotApproved.selector));
        router.finalizeSlashing(slashId);
    }

    function test_Revert_finalizeSlashing() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        // register operator
        vm.prank(operator);
        registry.registerAsOperator("operator", "Operator X");

        // create a vault for operator and fund it
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;
        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        vm.prank(owner);
        router.setVaultWhitelist(vault, true);
        underlying.mint(vault, 1_000_000 * underlyingMinorUnit); // mint 1m to the vault

        // register service and enable slashing
        vm.startPrank(service);
        registry.registerAsService("service", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: service, maxMbips: 1_000_000, resolutionWindow: 3600})
        );
        vm.stopPrank();

        // register service to operator and vice versa
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        vm.prank(service);
        registry.registerOperatorToService(operator);

        // enable slashing for operator
        vm.prank(operator);
        registry.approveSlashingFor(service);

        // advance time to allow slashing
        _advanceBlockBy(100);

        // Service initiates slashing request
        ISLAYRouterSlashingV2.Payload memory requestPayload = ISLAYRouterSlashingV2.Payload({
            mbips: 1_000_000, // 10%
            timestamp: uint32(block.timestamp) - 100,
            operator: operator,
            reason: "Missing Blocks"
        });
        vm.prank(service);
        bytes32 slashId = router.requestSlashing(requestPayload);

        // Revert when non-service tries to finalize slashing
        vm.prank(operator);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistryV2.ServiceNotFound.selector, operator));
        router.finalizeSlashing(slashId);

        // Revert when service tries to finalize slashing but is not the one who initiated the request
        address anotherService = makeAddr("Another Service");
        vm.startPrank(anotherService);
        registry.registerAsService("another-service", "Another Service");
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.Unauthorized.selector));
        router.finalizeSlashing(slashId);
        vm.stopPrank();

        // Revert when slashing request has not been locked
        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.InvalidStatus.selector));
        router.finalizeSlashing(slashId);

        // Service locks slashing
        _advanceBlockBySeconds(3600);
        vm.prank(service);
        router.lockSlashing(slashId);

        // Revert when slashing request has not been confirmed by guardrail
        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.GuardrailHaveNotApproved.selector));
        router.finalizeSlashing(slashId);

        // Guardrail votes on the slashing request
        address guardrail = makeAddr("Guardrail");
        vm.prank(owner);
        router.setGuardrail(guardrail);
        vm.prank(guardrail);
        vm.expectEmit();
        emit ISLAYRouterSlashingV2.GuardrailApproval(slashId, true);
        router.guardrailApprove(slashId, true);

        // Service finalizes slashing
        vm.prank(service);
        vm.expectEmit();
        emit ISLAYRouterSlashingV2.SlashingFinalized(service, operator, slashId, service);
        router.finalizeSlashing(slashId);

        // Revert when service tries to finalize slashing again
        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.InvalidStatus.selector));
        router.finalizeSlashing(slashId);
    }

    function test_Revert_guardrailConfirm() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        // register operator
        vm.prank(operator);
        registry.registerAsOperator("operator", "Operator X");

        // create a vault for operator and fund it
        MockERC20 underlying = new MockERC20("Token", "TKN", 18);
        uint8 underlyingDecimal = underlying.decimals();
        uint256 underlyingMinorUnit = 10 ** underlyingDecimal;
        vm.prank(operator);
        address vault = address(vaultFactory.create(underlying));
        vm.prank(owner);
        router.setVaultWhitelist(vault, true);
        underlying.mint(vault, 1_000_000 * underlyingMinorUnit); // mint 1m to the vault

        // register service and enable slashing
        vm.startPrank(service);
        registry.registerAsService("service", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: service, maxMbips: 1_000_000, resolutionWindow: 3600})
        );
        vm.stopPrank();

        // register service to operator and vice versa
        vm.prank(operator);
        registry.registerServiceToOperator(service);
        vm.prank(service);
        registry.registerOperatorToService(operator);

        // enable slashing for operator
        vm.prank(operator);
        registry.approveSlashingFor(service);

        // advance time to allow slashing
        _advanceBlockBy(100);

        // Service initiates slashing request
        ISLAYRouterSlashingV2.Payload memory requestPayload = ISLAYRouterSlashingV2.Payload({
            mbips: 1_000_000, // 10%
            timestamp: uint32(block.timestamp) - 100,
            operator: operator,
            reason: "Missing Blocks"
        });
        vm.prank(service);
        bytes32 slashId = router.requestSlashing(requestPayload);

        // fast forward to after resolution window
        _advanceBlockBySeconds(3600);

        // Service locks slashing
        vm.prank(service);
        router.lockSlashing(slashId);

        // Revert when guardrail has not been set
        address guardrail = makeAddr("Guardrail");
        vm.prank(guardrail);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.Unauthorized.selector));
        router.guardrailApprove(slashId, true);

        // owner sets the guardrail
        vm.prank(owner);
        router.setGuardrail(guardrail);

        // Revert when non-guardrail tries to confirm slashing
        address anotherGuardrail = makeAddr("Another Guardrail");
        vm.prank(anotherGuardrail);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.Unauthorized.selector));
        router.guardrailApprove(slashId, true);

        // Revert when guardrail tries to confirm slashing with invalid id
        bytes32 invalidSlashId = keccak256(abi.encodePacked("invalid"));
        vm.prank(guardrail);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.SlashingRequestNotFound.selector));
        router.guardrailApprove(invalidSlashId, true);

        // guardrail confirms the slashing request
        vm.prank(guardrail);
        vm.expectEmit();
        emit ISLAYRouterSlashingV2.GuardrailApproval(slashId, true);
        router.guardrailApprove(slashId, true);

        // Revert when guardrail tries to confirm slashing again
        vm.prank(guardrail);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.GuardrailAlreadyApproved.selector));
        router.guardrailApprove(slashId, true);

        // Revert when guardrail tries to change the confirm status on a slashId
        vm.prank(guardrail);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.GuardrailAlreadyApproved.selector));
        router.guardrailApprove(slashId, false);
    }

    function test_slashRequest_back_to_back() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        vm.prank(operator);
        registry.registerAsOperator("operator.com", "Operator X");

        vm.startPrank(service);
        registry.registerAsService("service.com", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: vm.randomAddress(), maxMbips: 100_000, resolutionWindow: 3600})
        );
        registry.registerOperatorToService(operator);
        vm.stopPrank();

        vm.prank(operator);
        registry.registerServiceToOperator(service);

        _advanceBlockBy(10);

        vm.prank(operator);
        registry.approveSlashingFor(service);

        uint32 newDelay = 8 days;
        vm.prank(operator);
        registry.setWithdrawalDelay(newDelay);

        _advanceBlockBy(10);

        uint32 timeAtWhichOffenseOccurs = uint32(block.timestamp);

        _advanceBlockBy(10);

        ISLAYRouterSlashingV2.Payload memory request = ISLAYRouterSlashingV2.Payload({
            mbips: 100,
            timestamp: timeAtWhichOffenseOccurs,
            operator: operator,
            reason: "Missing Blocks"
        });

        vm.prank(service);
        router.requestSlashing(request);

        vm.prank(service);
        ISLAYRouterSlashingV2.Payload memory request2 = ISLAYRouterSlashingV2.Payload({
            mbips: 200,
            timestamp: timeAtWhichOffenseOccurs,
            operator: operator,
            reason: "Double Signs"
        });
        vm.expectRevert("Previous slashing request lifecycle not completed");
        router.requestSlashing(request2);
    }

    function test_slashRequest_offense_at_future() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        vm.prank(operator);
        registry.registerAsOperator("operator.com", "Operator X");

        vm.startPrank(service);
        registry.registerAsService("service.com", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: vm.randomAddress(), maxMbips: 100_000, resolutionWindow: 3600})
        );
        registry.registerOperatorToService(operator);
        vm.stopPrank();

        vm.prank(operator);
        registry.registerServiceToOperator(service);

        _advanceBlockBy(10);

        vm.prank(operator);
        registry.approveSlashingFor(service);

        uint32 newDelay = 8 days;
        vm.prank(operator);
        registry.setWithdrawalDelay(newDelay);

        _advanceBlockBy(10);

        ISLAYRouterSlashingV2.Payload memory request = ISLAYRouterSlashingV2.Payload({
            mbips: 100,
            timestamp: uint32(block.timestamp + 12 * 2),
            operator: operator,
            reason: "Missing Blocks"
        });

        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterV2.TimestampInFuture.selector));
        router.requestSlashing(request);
    }

    function test_slashRequest_none_service() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        vm.prank(operator);
        registry.registerAsOperator("operator.com", "Operator X");

        vm.startPrank(service);
        registry.registerAsService("service.com", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: vm.randomAddress(), maxMbips: 100_000, resolutionWindow: 3600})
        );
        registry.registerOperatorToService(operator);
        vm.stopPrank();

        vm.prank(operator);
        registry.registerServiceToOperator(service);

        _advanceBlockBy(10);

        vm.prank(operator);
        registry.approveSlashingFor(service);

        uint32 newDelay = 8 days;
        vm.prank(operator);
        registry.setWithdrawalDelay(newDelay);

        _advanceBlockBy(10);

        ISLAYRouterSlashingV2.Payload memory request = ISLAYRouterSlashingV2.Payload({
            mbips: 100,
            timestamp: uint32(block.timestamp),
            operator: operator,
            reason: "Missing Blocks"
        });

        vm.prank(operator);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRegistryV2.ServiceNotFound.selector, address(operator)));
        router.requestSlashing(request);
    }

    function test_slashRequest_out_of_bounds() public {
        _advanceBlockBy(20000000);
        address operator = makeAddr("Operator X");
        address service = makeAddr("Service X");

        vm.prank(operator);
        registry.registerAsOperator("operator.com", "Operator X");

        vm.startPrank(service);
        registry.registerAsService("service.com", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: vm.randomAddress(), maxMbips: 100_00, resolutionWindow: 3600})
        );
        registry.registerOperatorToService(operator);
        vm.stopPrank();

        vm.prank(operator);
        registry.registerServiceToOperator(service);

        _advanceBlockBy(10);

        vm.prank(operator);
        registry.approveSlashingFor(service);

        uint32 newDelay = 8 days;
        vm.prank(operator);
        registry.setWithdrawalDelay(newDelay);

        _advanceBlockBy(10);

        ISLAYRouterSlashingV2.Payload memory request = ISLAYRouterSlashingV2.Payload({
            mbips: 100,
            timestamp: uint32(block.timestamp),
            operator: operator,
            reason: "Missing Blocks"
        });

        _advanceBlockBy(10000000);

        vm.prank(service);
        vm.expectRevert("timestamp too old");
        router.requestSlashing(request);

        ISLAYRouterSlashingV2.Payload memory request2 = ISLAYRouterSlashingV2.Payload({
            mbips: 100_000,
            timestamp: uint32(block.timestamp),
            operator: operator,
            reason: "Missing Blocks"
        });

        vm.prank(service);
        vm.expectRevert(abi.encodeWithSelector(ISLAYRouterSlashingV2.MbipsExceedsMaxAllowed.selector));
        router.requestSlashing(request2);
    }
}
