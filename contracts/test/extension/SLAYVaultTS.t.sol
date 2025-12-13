// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.24;

import {SLAYVaultTS} from "../../src/extension/SLAYVaultTS.sol";
import {SLAYERC20Derivative} from "../../src/extension/SLAYERC20Derivative.sol";
import {TestSuiteV2} from "../TestSuiteV2.sol";
import {Test} from "forge-std/Test.sol";
import {MockERC20} from "../MockERC20.sol";
import {ISLAYRegistryV2} from "../../src/interface/ISLAYRegistryV2.sol";
import {ISLAYRouterSlashingV2} from "../../src/interface/ISLAYRouterSlashingV2.sol";
import {SLAYSlashToken} from "../../src/extension/SLAYSlashToken.sol";

import {Vm} from "forge-std/Vm.sol";
import {console} from "forge-std/console.sol";

contract SLAYVaultTSTest is Test, TestSuiteV2 {
    MockERC20 public underlying = new MockERC20("Wrapped Bitcoin", "WBTC", 8);
    uint8 public underlyingDecimal = underlying.decimals();
    uint256 public underlyingMinorUnit = 10 ** underlyingDecimal;
    address public immutable operator = makeAddr("Operator Y");

    SLAYVaultTS public vaultTS;

    function setUp() public override {
        TestSuiteV2.setUp();

        vm.startPrank(operator);
        registry.registerAsOperator("https://example.com", "Operator Y");
        vm.stopPrank();

        // setup SLAYVaultTS
        vaultTS = new SLAYVaultTS(router, registry);
        vaultTS.initialize(underlying, operator, "SLAY vaultTS", "STS");

        // whitelist vault in router
        vm.startPrank(owner);
        router.setVaultWhitelist(address(vaultTS), true);
        vm.stopPrank();
    }

    function test_lockSlashing_withSlashToken() public {
        // Mint some underlying tokens to staker
        address staker = makeAddr("staker");
        underlying.mint(staker, 1_000 * underlyingMinorUnit);

        // staker deposits into vaultTS
        vm.startPrank(staker);
        underlying.approve(address(vaultTS), type(uint256).max);
        vaultTS.deposit(500 * underlyingMinorUnit, staker);
        vm.stopPrank();

        // slash lock called by router
        vm.recordLogs();
        vm.startPrank(address(router));
        vaultTS.lockSlashing(100 * underlyingMinorUnit);
        vm.stopPrank();

        // get recorded events
        Vm.Log[] memory logs = vm.getRecordedLogs();
        bytes32 slashTokenCreatedTopic = keccak256("SlashTokenCreated(address,uint32)");

        // get slash token address from event
        address slashTokenAddress;
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == slashTokenCreatedTopic) {
                slashTokenAddress = address(uint160(uint256(logs[i].topics[1])));
                break;
            }
        }

        SLAYERC20Derivative slashToken = SLAYERC20Derivative(slashTokenAddress);

        _advanceBlockBy(1); // move to next block to record checkpoint

        // check slash token balance of staker
        assertEq(slashToken.balanceOf(staker), 500 * underlyingMinorUnit, "Slash token balance should be 500");
    }

    function test_distributeRewardsWithSlashToken() public {
        _advanceBlockBy(1_000_000);
        // register service
        address service = makeAddr("service");
        vm.startPrank(service);
        registry.registerAsService("https://service.com", "Service A");
        registry.enableSlashing(
            ISLAYRegistryV2.SlashParameter({destination: service, maxMbips: 1_000_000, resolutionWindow: 3600})
        );
        // register operator to service
        registry.registerOperatorToService(operator);
        vm.stopPrank();

        // register service to operator
        vm.prank(operator);
        registry.registerServiceToOperator(service);

        _advanceBlockBy(1);

        // Mint some underlying tokens to stakers
        address staker = makeAddr("staker");
        address staker2 = makeAddr("staker2");
        address staker3 = makeAddr("staker3");
        underlying.mint(staker, 1_000 * underlyingMinorUnit);
        underlying.mint(staker2, 1_000 * underlyingMinorUnit);
        underlying.mint(staker3, 1_000 * underlyingMinorUnit);

        // staker deposits into vaultTS
        vm.startPrank(staker);
        underlying.approve(address(vaultTS), type(uint256).max);
        vaultTS.deposit(500 * underlyingMinorUnit, staker);
        vm.stopPrank();

        // staker2 deposits into vaultTS
        vm.startPrank(staker2);
        underlying.approve(address(vaultTS), type(uint256).max);
        vaultTS.deposit(300 * underlyingMinorUnit, staker2);
        vm.stopPrank();
        // staker3 deposits into vaultTS
        vm.startPrank(staker3);
        underlying.approve(address(vaultTS), type(uint256).max);
        vaultTS.deposit(200 * underlyingMinorUnit, staker3);
        vm.stopPrank();

        // advance block to simulate time passing
        _advanceBlockBy(100);

        // initiate slash
        ISLAYRouterSlashingV2.Payload memory payload = ISLAYRouterSlashingV2.Payload({
            operator: operator,
            mbips: 1_000_000,
            timestamp: uint32(block.timestamp) - 10,
            reason: "Missing Blocks"
        });
        vm.prank(service);
        bytes32 slashId = router.requestSlashing(payload);

        _advanceBlockBy(360);

        // slash lock called by router
        vm.recordLogs();
        vm.prank(service);
        router.lockSlashing(slashId);

        // get slash token address
        address slashToken = vaultTS.slashTokens(0);
        SLAYSlashToken slashTokenContract = SLAYSlashToken(slashToken);

        // assert all stakers have slash token balance
        assertEq(slashTokenContract.balanceOf(staker), 500 * underlyingMinorUnit, "Slash token balance should be 500");
        assertEq(slashTokenContract.balanceOf(staker2), 300 * underlyingMinorUnit, "Slash token balance should be 300");
        assertEq(slashTokenContract.balanceOf(staker3), 200 * underlyingMinorUnit, "Slash token balance should be 200");

        // NOTE: code down here will require changes to ISLAYRouterV2, these are just shown to simulate that change

        // simulate underlying asset being deposited back to the slashed vault and recipient of vault token is the slash token contract.
        vm.startPrank(address(router));
        underlying.approve(address(vaultTS), type(uint256).max);
        uint256 receiptTokenAmount = vaultTS.deposit(100 * underlyingMinorUnit, slashToken);
        vm.stopPrank();

        // call distribute rewards on slash token
        vm.prank(address(vaultTS));
        slashTokenContract.distributeRewards(receiptTokenAmount);

        // staker claims their rewards
        vm.prank(staker);
        uint256 claimAmount = slashTokenContract.claim(staker);
        // staker2 claims their rewards
        vm.prank(staker2);
        uint256 claimAmount2 = slashTokenContract.claim(staker2);
        // staker3 claims their rewards
        vm.prank(staker3);
        uint256 claimAmount3 = slashTokenContract.claim(staker3);

        // assert their vault receipt token have increased
        assertEq(vaultTS.balanceOf(staker), 55_555_555_555, "staker vault receipt token should be 55_555_555_555");
        assertEq(vaultTS.balanceOf(staker2), 33_333_333_333, "staker2 vault receipt token should be 33_333_333_333");
        assertEq(vaultTS.balanceOf(staker3), 22_222_222_222, "staker3 vault receipt token should be 22_222_222_222");

        // assert the vault total supply increased
        assertEq(vaultTS.totalSupply(), 111_111_111_111, "vault total supply should be 111_111_111_111");
        // assert the vault underlying balance remains the same
        assertEq(vaultTS.totalAssets(), 1_000 * underlyingMinorUnit, "vault underlying balance should be 1_000");
    }
}
