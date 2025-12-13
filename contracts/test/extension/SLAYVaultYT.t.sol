// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

import {SLAYVaultYT, SLAYPerpetualYieldToken, SLAYInverseYieldToken} from "../../src/extension/SLAYVaultYT.sol";
import {MockERC20} from "../MockERC20.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestSuiteV2} from "../TestSuiteV2.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {UnsafeUpgrades} from "@openzeppelin/foundry-upgrades/Upgrades.sol";
import {BeaconProxy} from "@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol";

import {console} from "forge-std/console.sol";

contract SLAYVaultYTTest is Test, TestSuiteV2 {
    SLAYVaultYT public vaultYT;
    SLAYPerpetualYieldToken public pyt;
    SLAYInverseYieldToken public ipt;

    MockERC20 public mstETH;

    address public operator;

    function setUp() public override {
        TestSuiteV2.setUp();

        operator = makeAddr("Operator");

        // create mock underlying asset
        mstETH = new MockERC20("Mock stETH", "mstETH", 6);

        // init vaultYT
        SLAYVaultYT vaultYTImpl = new SLAYVaultYT(router, registry);
        address beacon = UnsafeUpgrades.deployBeacon(address(vaultYTImpl), owner);

        bytes memory data =
            abi.encodeCall(SLAYVaultYT.initialize2, (IERC20(address(mstETH)), operator, "SLAY Vault YT", "SVYT"));
        BeaconProxy proxy = new BeaconProxy(beacon, data);

        vaultYT = SLAYVaultYT(address(proxy));

        pyt = vaultYT.perpetualYieldToken();
        ipt = vaultYT.inverseYieldToken();

        // whitelist vaultYT
        vm.prank(owner);
        router.setVaultWhitelist(address(vaultYT), true);
    }

    function test_depositAndMintPYT() public {
        address user1 = makeAddr("User1");

        // mint some underlying asset to user1
        mstETH.mint(user1, 10e6);

        // user1 deposit and mint PYT
        vm.startPrank(user1);
        mstETH.approve(address(vaultYT), 1e6);
        uint256 shares = vaultYT.depositAndMintPYT(1e6, user1);
        vm.stopPrank();

        assertEq(shares, 1e6, "Shares should be equal to deposited amount");
        assertEq(vaultYT.balanceOf(address(vaultYT)), 1e6, "Vault should have 1e6 shares");
        assertEq(pyt.balanceOf(user1), 1e6, "User1 should have 1e6 PYT");
        assertEq(ipt.balanceOf(user1), 1e6, "User1 should have 1e6 IPT");
    }

    /**
     * @dev Complex scenario with multiple users depositing, interest generation, and claiming interest
     * Flow:
     * 1. User1 deposits 1e6 assets mints PYT and IPT
     * 2. Simulate interest generation by depositing more assets directly to the vault
     * 3. User2 deposits 1e6 assets mints PYT and IPT
     * 4. Simulate more interest generation
     * 5. User3 deposits 1e6 assets mints PYT and IPT
     * 6. User1 claims interest in shares
     * 7. User2 claims interest in shares and mints more PYT from interest
     * 8. Simulate more interest generation
     * 9. Assert accrued interest for each user after the interest generation
     */
    function test_depositAndMintPYT_complex() public {
        address user1 = makeAddr("User1");
        address user2 = makeAddr("User2");
        address user3 = makeAddr("User3");

        // mint some underlying asset to users
        mstETH.mint(user1, 10e6);
        mstETH.mint(user2, 10e6);
        mstETH.mint(user3, 10e6);

        // user1 deposit and mint PYT
        vm.startPrank(user1);
        mstETH.approve(address(vaultYT), 1e6);
        uint256 shares = vaultYT.depositAndMintPYT(1e6, user1);
        vm.stopPrank();

        assertEq(shares, 1e6, "Shares should be equal to deposited amount");
        assertEq(vaultYT.balanceOf(address(vaultYT)), 1e6, "Vault should have 1e6 shares");
        assertEq(pyt.balanceOf(user1), 1e6, "User1 should have 1e6 PYT");
        assertEq(ipt.balanceOf(user1), 1e6, "User1 should have 1e6 IPT");

        // simulate interest generation by depositing more underlying asset directly to the vault
        mstETH.mint(address(vaultYT), 1e6);

        // user 2 deposit and mint PYT
        vm.startPrank(user2);
        mstETH.approve(address(vaultYT), 1e6);
        uint256 pyt2 = vaultYT.depositAndMintPYT(1e6, user2);
        vm.stopPrank();

        assertEq(pyt2, 999_999, "pyt and iyt amount should be 999_999");
        assertEq(pyt.balanceOf(user2), pyt2, "User2 should have 999_999 PYT");
        assertEq(ipt.balanceOf(user2), pyt2, "User2 should have 999_999 IPT");
        assertEq(vaultYT.balanceOf(address(vaultYT)), 1_500_000, "Vault should have total shares");

        // simulate more interest generation by depositing more underlying asset directly to the vault
        mstETH.mint(address(vaultYT), 1.5e6);

        // user 3 deposit and mint PYT
        vm.startPrank(user3);
        mstETH.approve(address(vaultYT), 1e6);
        uint256 pyt3 = vaultYT.depositAndMintPYT(1e6, user3);
        vm.stopPrank();

        assertEq(pyt3, 999_998, "pyt and iyt amount should be 999_998");
        assertEq(pyt.balanceOf(user3), 999_998, "User3 should have 999_998 PYT");
        assertEq(ipt.balanceOf(user3), 999_998, "User3 should have 999_998 IPT");
        assertEq(vaultYT.balanceOf(address(vaultYT)), 1_833_333, "Vault should have total shares");

        // user 1 claim interest in shares
        vm.startPrank(user1);
        uint256 interest1_preview = vaultYT.getAccruedInterest(user1);
        uint256 interest1 = vaultYT.claimInterest(user1);
        vm.stopPrank();

        assertEq(interest1_preview, interest1, "Previewed interest should match claimed interest");
        assertEq(interest1, 666_666, "User1 should have 666_666 (= 1e6 * 2/3 ) interest");
        assertEq(vaultYT.balanceOf(user1), 666_666, "User1 should have 666_666 shares left");

        // user 2 claim interest in shares
        vm.prank(user2);
        uint256 interest2 = vaultYT.claimInterest(user2);

        assertEq(interest2, 166_666, "User2 should have 166_666 (= 1e6 * 1/6 ) interest");
        assertEq(vaultYT.balanceOf(user2), 166_666, "User2 should have 166_666 shares left");

        // user 2 mint more PYT from interest
        vm.prank(user2);
        uint256 pyt2_2 = vaultYT.mintPYT(166_666, user2);

        // E(t) = 3
        assertEq(pyt2_2, 499_997, "pyt and iyt amount should be 499_997");
        assertEq(pyt.balanceOf(user2), 1_499_996, "User2 should have 1_499_996 PYT");
        assertEq(ipt.balanceOf(user2), 1_499_996, "User2 should have 1_499_996 IPT");
        assertEq(vaultYT.balanceOf(address(vaultYT)), 1_166_667, "Vault should have total shares");

        // simulate more interest generation by depositing more underlying asset directly to the vault
        mstETH.mint(address(vaultYT), 5.5e6);

        // assert accrued interest for each user after the interest generation
        uint256 interest1_after = vaultYT.getAccruedInterest(user1);
        uint256 interest2_after = vaultYT.getAccruedInterest(user2);
        uint256 interest3_after = vaultYT.getAccruedInterest(user3);

        // E(t') = 3
        // E(t) = 6

        // user1 have 1e6 pyt, so i(t) = 1e6 * (1/3 - 1/6) = 166_666
        assertEq(interest1_after, 166_666, "User1 should have 166_666 interest");

        // user2 have 1_499_996 pyt, so i(t) = 1_499_996 * (1/3 - 1/6) = 249_999
        assertEq(interest2_after, 249_999, "User2 should have 249_999 interest");

        // user3 have 999_998 pyt, so i(t) = 999_998 * (1/3 - 1/6) = 166_666
        assertEq(interest3_after, 166_666, "User3 should have 149_999 interest");
    }
}
