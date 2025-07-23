// SPDX-License-Identifier: BUSL-1.1
pragma solidity ^0.8.0;

import "../src/interface/ISLAYRewardsV2.sol";
import "./MockERC20.sol";
import "./TestSuiteV2.sol";
import {Test, console} from "forge-std/Test.sol";
import {IERC20Errors} from "@openzeppelin/contracts/interfaces/draft-IERC6093.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

contract SLAYRewardsV2Test is Test, TestSuiteV2 {
    function setUp() public override {
        TestSuiteV2.setUp();
    }

    // setup with a mock distribute.json reward
    // {
    //  "earners": [
    //    {
    //      "earner": "0x39a429c90c0033A102017bFAEb76527eBD9B3FEa",
    //      "reward": "100000000000000000"
    //    },
    //    {
    //      "earner": "0x911E794a6E79712B1d958821f70EE8882f714906",
    //      "reward": "300000000000000000"
    //    },
    //    {
    //      "earner": "0xd9322C1D287ef984dDF049caBABAB1a5b2E85cB3",
    //      "reward": "500000000000000000"
    //    },
    //    {
    //      "earner": "0x86d6Fda2f439537da03a5b76D5aE26412F4c4235",
    //      "reward": "200000000000000000"
    //    },
    //    {
    //      "earner": "0x0D3a5Ec49d5D92c318075eDB801C2A3699e9f201",
    //      "reward": "150000000000000000"
    //    }
    //  ]
    //}

    MockERC20 public rewardToken = new MockERC20("Wrapped Bitcoin", "WBTC", 8);
    uint256 public rewardTokenMinorUnit = 10 ** rewardToken.decimals();
    address public service = makeAddr("service");

    bytes32 internal merkleRoot =
        bytes32(abi.encodePacked(hex"2016f97ae135385b6942e4aa35c97bdcfdd599c9ddcd750868f8366173d58d3c"));

    function test_distributeRewards() public {
        // mint some rewards tokens to the service
        rewardToken.mint(service, 12_500_000_000 * rewardTokenMinorUnit); // 12.6 Billion WBTC

        // service distributes rewards
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 12_500_000_000 * rewardTokenMinorUnit);
        vm.expectEmit();
        emit ISLAYRewardsV2.RewardsDistributed(
            service, address(rewardToken), 12_500_000_000 * rewardTokenMinorUnit, merkleRoot
        );
        rewards.distributeRewards(address(rewardToken), 12_500_000_000 * rewardTokenMinorUnit, merkleRoot);
        vm.stopPrank();

        // check that the rewards were distributed correctly
        ISLAYRewardsV2.DistributionRoots memory roots = rewards.getDistributionRoots(service, address(rewardToken));
        assertEq(roots.prevRoot, bytes32(0), "Previous root should be zero");
        assertEq(roots.currentRoot, merkleRoot, "Current root should match the distributed merkle root");

        // check that the balance of the service,token pair is correct
        uint256 serviceTokenBalance = rewards.getBalance(service, address(rewardToken));
        assertEq(
            serviceTokenBalance,
            12_500_000_000 * rewardTokenMinorUnit,
            "Service balance should match the distributed amount"
        );

        // check that the service rewardToken balance is reduced
        uint256 serviceBalance = rewardToken.balanceOf(service);
        assertEq(serviceBalance, 0, "Service balance should be zero after distribution");

        // check that the rewards contract rewardToken balance is increased
        uint256 rewardsContractBalance = rewardToken.balanceOf(address(rewards));
        assertEq(
            rewardsContractBalance,
            12_500_000_000 * rewardTokenMinorUnit,
            "Rewards contract balance should match the distributed amount"
        );
    }

    function test_revert_distributeRewards_withZeroAllowance() public {
        // mint some rewards tokens to the service
        rewardToken.mint(service, 12_500_000_000 * rewardTokenMinorUnit); // 12.5 Billion WBTC

        // service distributes rewards
        vm.prank(service);
        vm.expectRevert(
            abi.encodeWithSelector(
                IERC20Errors.ERC20InsufficientAllowance.selector,
                address(rewards),
                0,
                12_500_000_000 * rewardTokenMinorUnit
            )
        );
        rewards.distributeRewards(address(rewardToken), 12_500_000_000 * rewardTokenMinorUnit, merkleRoot);
    }

    function test_revert_distributeRewards_withEmptyMerkleRoot() public {
        // mint some rewards tokens to the service
        rewardToken.mint(service, 12_500_000_000 * rewardTokenMinorUnit); // 12.5 Billion WBTC

        bytes32 emptyMerkleRoot = bytes32(0); // empty merkle root

        // service distributes rewards
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 12_500_000_000 * rewardTokenMinorUnit);
        vm.expectRevert("Merkle root cannot be empty");
        rewards.distributeRewards(address(rewardToken), 12_500_000_000 * rewardTokenMinorUnit, emptyMerkleRoot);
        vm.stopPrank();
    }

    function test_revert_distributeRewards_withInvalidToken() public {
        // mint some rewards tokens to the service
        rewardToken.mint(service, 12_500_000_000 * rewardTokenMinorUnit); // 12.5 Billion WBTC

        // service distributes rewards with empty token address
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 12_500_000_000 * rewardTokenMinorUnit);
        vm.expectRevert("Token address cannot be zero");
        rewards.distributeRewards(address(0x0), 12_500_000_000 * rewardTokenMinorUnit, merkleRoot);
        vm.stopPrank();

        // service distributes rewards with invalid token address
        address invalidAddress = makeAddr("invalidToken");
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 12_500_000_000 * rewardTokenMinorUnit);
        vm.expectRevert(abi.encodeWithSelector(SafeERC20.SafeERC20FailedOperation.selector, address(invalidAddress)));
        rewards.distributeRewards(invalidAddress, 12_500_000_000 * rewardTokenMinorUnit, merkleRoot);
        vm.stopPrank();
    }

    function test_claimRewards() public {
        // mint some rewards tokens to the service
        rewardToken.mint(service, 12_500_000_000 * rewardTokenMinorUnit); // 12.5 Billion WBTC

        // service distributes rewards
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 12_500_000_000 * rewardTokenMinorUnit);
        rewards.distributeRewards(address(rewardToken), 12_500_000_000 * rewardTokenMinorUnit, merkleRoot);
        vm.stopPrank();

        address earner = address(0x86d6Fda2f439537da03a5b76D5aE26412F4c4235);

        bytes32[] memory proof = new bytes32[](3);
        proof[0] = bytes32(0xc5d11bcf5b13a6839acbf0f57fe1b202fe159e5b5b3bbbd3b9dd1a69e1aa84dc);
        proof[1] = bytes32(0x8d25a6cb91e258d097872c7e37477e311da5fcd048037a7d729d9eac13903882);
        proof[2] = bytes32(0x8a08f27e959995b62300cc7b9cdebb565e9ba6c0bfabf76c58da0c98ac378e81);

        // create a claimable proof for the first earner
        ISLAYRewardsV2.ClaimableRewardProof memory claimRewardsParams = ISLAYRewardsV2.ClaimableRewardProof({
            service: service,
            token: address(rewardToken),
            amount: 2_000_000_000 * rewardTokenMinorUnit, // 2 billion WBTC,
            recipient: earner,
            merkleRoot: merkleRoot,
            proof: proof,
            leafIndex: 3,
            totalLeaves: 5
        });

        // claim the rewards
        vm.prank(earner);
        vm.expectEmit();
        emit ISLAYRewardsV2.RewardsClaimed(
            service, address(rewardToken), earner, earner, 2_000_000_000 * rewardTokenMinorUnit, merkleRoot
        );
        rewards.claimRewards(claimRewardsParams);

        // check that the claimed amount is correct
        uint256 claimedAmount = rewards.getClaimedRewards(service, address(rewardToken), earner);
        assertEq(claimedAmount, 2_000_000_000 * rewardTokenMinorUnit, "Claimed amount should match the claimed amount");

        // check that the balance of the contract is reduced
        uint256 contractBalance = rewardToken.balanceOf(address(rewards));
        assertEq(
            contractBalance,
            (12_500_000_000 - 2_000_000_000) * rewardTokenMinorUnit,
            "Contract balance should be reduced by the claimed amount"
        );

        // check that the recipient's balance is increased
        uint256 recipientBalance = rewardToken.balanceOf(address(earner));
        assertEq(
            recipientBalance, 2_000_000_000 * rewardTokenMinorUnit, "Recipient balance should match the claimed amount"
        );
    }

    function test_revert_claimRewards_withInvalidMerkleRoot() public {
        // mint some rewards tokens to the service
        rewardToken.mint(service, 12_500_000_000 * rewardTokenMinorUnit); // 12.5 Billion WBTC

        // service distributes rewards
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 12_500_000_000 * rewardTokenMinorUnit);
        rewards.distributeRewards(address(rewardToken), 12_500_000_000 * rewardTokenMinorUnit, merkleRoot);
        vm.stopPrank();

        address earner = address(0x86d6Fda2f439537da03a5b76D5aE26412F4c4235);

        bytes32[] memory proof = new bytes32[](3);
        proof[0] = bytes32(0xc5d11bcf5b13a6839acbf0f57fe1b202fe159e5b5b3bbbd3b9dd1a69e1aa84dc);
        proof[1] = bytes32(0x8d25a6cb91e258d097872c7e37477e311da5fcd048037a7d729d9eac13903882);
        proof[2] = bytes32(0x8a08f27e959995b62300cc7b9cdebb565e9ba6c0bfabf76c58da0c98ac378e81);

        bytes32 emptyMerkleRoot = bytes32(0); // empty merkle root

        // create a claimable proof for the first earner with an empty merkle root
        ISLAYRewardsV2.ClaimableRewardProof memory claimRewardsParams = ISLAYRewardsV2.ClaimableRewardProof({
            service: service,
            token: address(rewardToken),
            amount: 2_000_000_000 * rewardTokenMinorUnit, // 2 billion WBTC,
            recipient: earner,
            merkleRoot: emptyMerkleRoot, // empty merkle root
            proof: proof,
            leafIndex: 3,
            totalLeaves: 5
        });

        // try to claim the rewards and expect revert
        vm.prank(earner);
        vm.expectRevert("Merkle root cannot be empty");
        rewards.claimRewards(claimRewardsParams);

        bytes32 invalidMerkleRoot =
            bytes32(abi.encodePacked(hex"1c8dd9ca252d7eb9bf1cccb9ab587e9a1dccca4c7474bb8739c0e5218964a2b4"));
        // create a claimable proof for the first earner with an invalid merkle root
        claimRewardsParams.merkleRoot = bytes32(invalidMerkleRoot);
        // try to claim the rewards and expect revert
        vm.prank(earner);
        vm.expectRevert(
            abi.encodeWithSelector(
                ISLAYRewardsV2.InvalidMerkleRoot.selector, service, address(rewardToken), bytes32(invalidMerkleRoot)
            )
        );
        rewards.claimRewards(claimRewardsParams);
    }

    function test_revert_claimRewards_withAmountAlreadyClaimed() public {
        // mint some rewards tokens to the service
        rewardToken.mint(service, 13_500_000_000 * rewardTokenMinorUnit); // 13.5 Billion WBTC

        // service distributes rewards
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 12_500_000_000 * rewardTokenMinorUnit);
        rewards.distributeRewards(address(rewardToken), 12_500_000_000 * rewardTokenMinorUnit, merkleRoot);
        vm.stopPrank();

        address earner = address(0x86d6Fda2f439537da03a5b76D5aE26412F4c4235);

        bytes32[] memory proof = new bytes32[](3);
        proof[0] = bytes32(0xc5d11bcf5b13a6839acbf0f57fe1b202fe159e5b5b3bbbd3b9dd1a69e1aa84dc);
        proof[1] = bytes32(0x8d25a6cb91e258d097872c7e37477e311da5fcd048037a7d729d9eac13903882);
        proof[2] = bytes32(0x8a08f27e959995b62300cc7b9cdebb565e9ba6c0bfabf76c58da0c98ac378e81);

        // create a claimable proof for the earner
        ISLAYRewardsV2.ClaimableRewardProof memory claimRewardsParams = ISLAYRewardsV2.ClaimableRewardProof({
            service: service,
            token: address(rewardToken),
            amount: 2_000_000_000 * rewardTokenMinorUnit, // 2 billion WBTC,
            recipient: earner,
            merkleRoot: merkleRoot,
            proof: proof,
            leafIndex: 3,
            totalLeaves: 5
        });

        // claim the first time successfully
        vm.prank(earner);
        rewards.claimRewards(claimRewardsParams);

        // service updates merkle tree with new rewards, increasing the earner's rewards by 1 billion WBTC
        bytes32 updatedMerkleRoot =
            bytes32(abi.encodePacked(hex"cb93318a2eabbb3cfdd90a540dfbda2671bc09c701ae1c9b916ca41829a42e32"));
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 1_000_000_000 * rewardTokenMinorUnit);
        rewards.distributeRewards(address(rewardToken), 1_000_000_000 * rewardTokenMinorUnit, updatedMerkleRoot);
        vm.stopPrank();

        // create a new claimable proof for the earner with the updated rewards
        claimRewardsParams.merkleRoot = updatedMerkleRoot;
        claimRewardsParams.amount = 2_000_000_000 * rewardTokenMinorUnit; // 2 billion WBTC (outdated value)

        // try to claim the rewards again and expect revert
        vm.prank(earner);
        vm.expectRevert(
            abi.encodeWithSelector(
                ISLAYRewardsV2.AmountAlreadyClaimed.selector,
                service,
                address(rewardToken),
                earner,
                2_000_000_000 * rewardTokenMinorUnit
            )
        );
        rewards.claimRewards(claimRewardsParams);
    }

    // init struct to prevent stack too deep errors
    struct TestLifecycleVars {
        address earner1;
        address earner2;
        address earner3;
        bytes32 firstMerkleRoot;
        bytes32 secondMerkleRoot;
    }

    TestLifecycleVars public tl_vars;

    /**
     * Test the lifecycle of the SLAYRewardsV2 contract.
     * This test will cover the following steps:
     *  1. Service distribute rewards for the first time
     *  2. Earner1 claims rewards
     *  3. Service updates the rewards distribution with a new Merkle root
     *  4. Earner2 should be able to claim rewards with the first merkle root
     *  5. Earner3 claims rewards ( should receive amount from the new Merkle root )
     *  6. Earner1 claims rewards again ( should receive the difference between the new and old Merkle root )
     */
    function test_lifecycle() public {
        // mint some rewards tokens to the service
        rewardToken.mint(service, 11_500_000_000 * rewardTokenMinorUnit); // 11.5 Billion WBTC

        // first merkle root should correspond to
        // {
        //  "earners": [
        //    {
        //      "earner": "0x39a429c90c0033A102017bFAEb76527eBD9B3FEa",
        //      "reward": "100000000000000000"
        //    },
        //    {
        //      "earner": "0x911E794a6E79712B1d958821f70EE8882f714906",
        //      "reward": "300000000000000000"
        //    },
        //    {
        //      "earner": "0xd9322C1D287ef984dDF049caBABAB1a5b2E85cB3",
        //      "reward": "500000000000000000"
        //    }
        //  ]
        //}
        tl_vars.firstMerkleRoot =
            bytes32(abi.encodePacked(hex"b016f17df3c3aac2813e7c7c4bc749f38a68466cad1720aff1df0ff9ba6bfaab"));

        tl_vars.earner1 = address(0x39a429c90c0033A102017bFAEb76527eBD9B3FEa);
        tl_vars.earner2 = address(0x911E794a6E79712B1d958821f70EE8882f714906);
        tl_vars.earner3 = address(0xd9322C1D287ef984dDF049caBABAB1a5b2E85cB3);

        // 1. service distributes rewards
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 9_000_000_000 * rewardTokenMinorUnit);
        vm.expectEmit();
        emit ISLAYRewardsV2.RewardsDistributed(
            service, address(rewardToken), 9_000_000_000 * rewardTokenMinorUnit, tl_vars.firstMerkleRoot
        );
        rewards.distributeRewards(address(rewardToken), 9_000_000_000 * rewardTokenMinorUnit, tl_vars.firstMerkleRoot);
        vm.stopPrank();

        // check that the rewards were distributed correctly
        ISLAYRewardsV2.DistributionRoots memory roots = rewards.getDistributionRoots(service, address(rewardToken));
        assertEq(roots.prevRoot, bytes32(0), "Previous root should be zero");
        assertEq(roots.currentRoot, tl_vars.firstMerkleRoot, "Current root should match the distributed merkle root");

        // 2. earner1 claims rewards
        bytes32[] memory proof1 = new bytes32[](2);
        proof1[0] = bytes32(0xfcea6c7ebfa548d53603d8c1297ca2b50965faf289892fa72221569a59c64a22);
        proof1[1] = bytes32(0x40da4a5e672f95c4271b7d47b118de0d9a524bae94c57489eaff4c4b27cd4e71);
        ISLAYRewardsV2.ClaimableRewardProof memory claimRewardsParams1 = ISLAYRewardsV2.ClaimableRewardProof({
            service: service,
            token: address(rewardToken),
            amount: 1_000_000_000 * rewardTokenMinorUnit, // 1 billion WBTC
            recipient: tl_vars.earner1,
            merkleRoot: tl_vars.firstMerkleRoot,
            proof: proof1,
            leafIndex: 0,
            totalLeaves: 3
        });
        vm.prank(tl_vars.earner1);
        vm.expectEmit();
        emit ISLAYRewardsV2.RewardsClaimed(
            service,
            address(rewardToken),
            tl_vars.earner1,
            tl_vars.earner1,
            1_000_000_000 * rewardTokenMinorUnit,
            tl_vars.firstMerkleRoot
        );
        rewards.claimRewards(claimRewardsParams1);

        // check that the claimed amount is correct
        uint256 claimedAmount1 = rewards.getClaimedRewards(service, address(rewardToken), tl_vars.earner1);
        assertEq(claimedAmount1, 1_000_000_000 * rewardTokenMinorUnit, "Claimed amount should match the claimed amount");
        // check that the recipient's balance is increased
        uint256 recipientBalance1 = rewardToken.balanceOf(tl_vars.earner1);
        assertEq(
            recipientBalance1, 1_000_000_000 * rewardTokenMinorUnit, "Recipient balance should match the claimed amount"
        );

        // service updates the rewards distribution with a new Merkle root which correspond with
        // {
        //  "earners": [
        //    {
        //      "earner": "0x39a429c90c0033A102017bFAEb76527eBD9B3FEa",
        //      "reward": "150000000000000000"
        //    },
        //    {
        //      "earner": "0x911E794a6E79712B1d958821f70EE8882f714906",
        //      "reward": "400000000000000000"
        //    },
        //    {
        //      "earner": "0xd9322C1D287ef984dDF049caBABAB1a5b2E85cB3",
        //      "reward": "600000000000000000"
        //    }
        //  ]
        //}
        tl_vars.secondMerkleRoot =
            bytes32(abi.encodePacked(hex"e01fed86b0ac968f2495c422c7057214263c6f0b775965fa611b39549394f27c"));

        // 3. service distributes second rewards
        vm.startPrank(service);
        rewardToken.approve(address(rewards), 2_500_000_000 * rewardTokenMinorUnit); // 2.5 Billion WBTC
        vm.expectEmit();
        emit ISLAYRewardsV2.RewardsDistributed(
            service, address(rewardToken), 2_500_000_000 * rewardTokenMinorUnit, tl_vars.secondMerkleRoot
        );
        rewards.distributeRewards(address(rewardToken), 2_500_000_000 * rewardTokenMinorUnit, tl_vars.secondMerkleRoot);
        vm.stopPrank();

        // check that the rewards were distributed correctly
        ISLAYRewardsV2.DistributionRoots memory roots2 = rewards.getDistributionRoots(service, address(rewardToken));
        assertEq(roots2.prevRoot, tl_vars.firstMerkleRoot, "Previous root should match the first merkle root");
        assertEq(roots2.currentRoot, tl_vars.secondMerkleRoot, "Current root should match the second merkle root");

        // 4. earner2 claims rewards using the first merkle root
        bytes32[] memory proof2 = new bytes32[](2);
        proof2[0] = bytes32(0x2ca7299d66e56c05cddd0e38699dc218b9e2ee1ea55d4f19837f8fb82dbd81cb);
        proof2[1] = bytes32(0x40da4a5e672f95c4271b7d47b118de0d9a524bae94c57489eaff4c4b27cd4e71);
        ISLAYRewardsV2.ClaimableRewardProof memory claimRewardsParams2 = ISLAYRewardsV2.ClaimableRewardProof({
            service: service,
            token: address(rewardToken),
            amount: 3_000_000_000 * rewardTokenMinorUnit, // 3 billion WBTC
            recipient: tl_vars.earner2,
            merkleRoot: tl_vars.firstMerkleRoot,
            proof: proof2,
            leafIndex: 1,
            totalLeaves: 3
        });
        vm.prank(tl_vars.earner2);
        vm.expectEmit();
        emit ISLAYRewardsV2.RewardsClaimed(
            service,
            address(rewardToken),
            tl_vars.earner2,
            tl_vars.earner2,
            3_000_000_000 * rewardTokenMinorUnit,
            tl_vars.firstMerkleRoot
        );
        rewards.claimRewards(claimRewardsParams2);
        // check that the claimed amount is correct
        uint256 claimedAmount2 = rewards.getClaimedRewards(service, address(rewardToken), tl_vars.earner2);
        assertEq(claimedAmount2, 3_000_000_000 * rewardTokenMinorUnit, "Claimed amount should match the claimed amount");
        // check that the recipient's balance is increased
        uint256 recipientBalance2 = rewardToken.balanceOf(tl_vars.earner2);
        assertEq(
            recipientBalance2, 3_000_000_000 * rewardTokenMinorUnit, "Recipient balance should match the claimed amount"
        );

        // 5. earner3 claims rewards using the second merkle root
        bytes32[] memory proof3 = new bytes32[](2);
        proof3[0] = bytes32(0x0000000000000000000000000000000000000000000000000000000000000000);
        proof3[1] = bytes32(0xbb394d2efc15e45004f10f3c815fb5cf9870aabc0b064b4b23f8861ad2f3cfed);
        ISLAYRewardsV2.ClaimableRewardProof memory claimRewardsParams3 = ISLAYRewardsV2.ClaimableRewardProof({
            service: service,
            token: address(rewardToken),
            amount: 6_000_000_000 * rewardTokenMinorUnit, // 6 billion WBTC
            recipient: tl_vars.earner3,
            merkleRoot: tl_vars.secondMerkleRoot,
            proof: proof3,
            leafIndex: 2,
            totalLeaves: 3
        });
        vm.prank(tl_vars.earner3);
        vm.expectEmit();
        emit ISLAYRewardsV2.RewardsClaimed(
            service,
            address(rewardToken),
            tl_vars.earner3,
            tl_vars.earner3,
            6_000_000_000 * rewardTokenMinorUnit,
            tl_vars.secondMerkleRoot
        );
        rewards.claimRewards(claimRewardsParams3);
        // check that the claimed amount is correct
        uint256 claimedAmount3 = rewards.getClaimedRewards(service, address(rewardToken), tl_vars.earner3);
        assertEq(claimedAmount3, 6_000_000_000 * rewardTokenMinorUnit, "Claimed amount should match the claimed amount");
        // check that the recipient's balance is increased
        uint256 recipientBalance3 = rewardToken.balanceOf(tl_vars.earner3);
        assertEq(
            recipientBalance3, 6_000_000_000 * rewardTokenMinorUnit, "Recipient balance should match the claimed amount"
        );

        // 6. earner1 claims rewards again ( should receive the difference between the new and old Merkle root )
        bytes32[] memory proof4 = new bytes32[](2);
        proof4[0] = bytes32(0x33e292a8b1a6b8db8f87780db5cbd57234f81daafa95077db68d6edef27cbfdc);
        proof4[1] = bytes32(0x981a103b03f593ecae5fde2836141dc43becb1e7a758b2e0609e0f91b204d543);
        ISLAYRewardsV2.ClaimableRewardProof memory claimRewardsParams4 = ISLAYRewardsV2.ClaimableRewardProof({
            service: service,
            token: address(rewardToken),
            amount: 1_500_000_000 * rewardTokenMinorUnit, // 1.5 billion WBTC
            recipient: tl_vars.earner1,
            merkleRoot: tl_vars.secondMerkleRoot,
            proof: proof4,
            leafIndex: 0,
            totalLeaves: 3
        });
        vm.prank(tl_vars.earner1);
        vm.expectEmit();
        emit ISLAYRewardsV2.RewardsClaimed(
            service,
            address(rewardToken),
            tl_vars.earner1,
            tl_vars.earner1,
            500_000_000 * rewardTokenMinorUnit, // should receive 500 million WBTC (1.5 - 1)
            tl_vars.secondMerkleRoot
        );
        rewards.claimRewards(claimRewardsParams4);
        // check that the claimed amount is correct
        uint256 claimedAmount4 = rewards.getClaimedRewards(service, address(rewardToken), tl_vars.earner1);
        assertEq(claimedAmount4, 1_500_000_000 * rewardTokenMinorUnit, "Claimed amount should match the claimed amount");
        // check that the recipient's balance is increased
        uint256 recipientBalance4 = rewardToken.balanceOf(tl_vars.earner1);
        assertEq(
            recipientBalance4, 1_500_000_000 * rewardTokenMinorUnit, "Recipient balance should match the claimed amount"
        );
        // check that the balance of the contract is reduced
        uint256 contractBalance = rewardToken.balanceOf(address(rewards));
        assertEq(
            contractBalance,
            1_000_000_000 * rewardTokenMinorUnit, // 11.5 - 1 - 3 - 6 - 0.5 = 1 Billion WBTC
            "Contract balance should be reduced by the claimed amounts"
        );
    }
}
