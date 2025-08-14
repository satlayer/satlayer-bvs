import { beforeEach, describe, expect, test, vi } from "vitest";
import { PolicyManager } from "./policy-manager";
import { Api } from "./api";
import {
  AnvilContainer,
  ChainName,
  EVMContracts,
  mockerc20,
  StartedAnvilContainer,
  SuperTestClient,
} from "@satlayer/testcontainers";
import { execa } from "execa";
import { Account, getContract, GetContractReturnType, parseEther } from "viem";
import slayvault from "@satlayer/contracts/out/SLAYVaultV2.sol/SLAYVaultV2.json";

let policyManager: PolicyManager;
let api: Api;
let started: StartedAnvilContainer;
let contracts: EVMContracts;

let vaultAddress: `0x${string}`;
let serviceAccount: Account;
let operatorAccount: Account;
let stakerAccount: Account;
let guardrailAccount: Account;

let vaultContract: GetContractReturnType<typeof slayvault.abi, SuperTestClient, `0x${string}`>;
let underlyingAssetContract: GetContractReturnType<typeof mockerc20.abi, SuperTestClient, `0x${string}`>;

describe("PolicyManager", () => {
  beforeEach(async () => {
    // Set up Anvil Container with EVMContracts bootstrapped
    started = await new AnvilContainer({
      forkedChainName: ChainName.EthereumMainnet,
    }).start();
    contracts = await EVMContracts.bootstrap(started);

    // create 3 accounts: operator, staker, service, and guardrail
    operatorAccount = started.generateAccount("operator");
    stakerAccount = started.generateAccount("staker");
    serviceAccount = started.generateAccount("service");
    // typically guardrail will be a multi-sig wallet, but for simplicity we use a single account
    guardrailAccount = started.generateAccount("guardrail");

    // Fund all 4 accounts with some gas token
    await started.setBalance(operatorAccount.address, parseEther("1"));
    await started.setBalance(stakerAccount.address, parseEther("1"));
    await started.setBalance(serviceAccount.address, parseEther("1"));
    await started.setBalance(guardrailAccount.address, parseEther("1"));

    // Operator register into SLAYRegistry
    await contracts.registry.write.registerAsOperator(["https://operator.com", "Operator"], {
      account: operatorAccount.address,
    });

    // Operator creates a vault with an underlying asset
    let underlyingAsset = await contracts.initERC20({ name: "wbtc", symbol: "WBTC", decimals: 8 });
    underlyingAssetContract = getContract({
      address: underlyingAsset.contractAddress,
      abi: mockerc20.abi,
      client: started.getClient(),
    });
    vaultAddress = await contracts.initVault({
      operator: operatorAccount,
      underlyingAsset: underlyingAsset.contractAddress,
    });
    vaultContract = getContract({
      address: vaultAddress,
      abi: slayvault.abi,
      client: started.getClient(),
    });

    // whitelist vault in SLAYRouter
    await contracts.router.write.setVaultWhitelist([vaultAddress, true]);

    // set guardrail in SLAYRouter
    await contracts.router.write.setGuardrail([guardrailAccount.address]);

    // Initialize Client
    api = new Api({
      startedContainer: started,
      vault: vaultAddress,
      registry: contracts.registry.address,
      router: contracts.router.address,
      operator: operatorAccount.address,
      service: serviceAccount.address,
      rewards: contracts.rewards.address,
    });

    // mint underlying asset to staker
    await underlyingAssetContract.write.mint([stakerAccount.address, 100_000_000n]);

    // Staker deposit into vault bank
    let depositAmount = 100_000_000n; // 100_000_000 underlying asset
    await underlyingAssetContract.write.approve([vaultContract.address, depositAmount], { account: stakerAccount });
    await vaultContract.write.deposit([depositAmount, stakerAccount.address], { account: stakerAccount });
    await started.mineBlock(1); // mine a block to ensure the deposit is processed

    // Initialize PolicyManager - set rewards token same as vault underlying asset
    policyManager = new PolicyManager(
      api,
      serviceAccount.address,
      operatorAccount.address,
      underlyingAsset.contractAddress,
      underlyingAsset.contractAddress,
    );
    await policyManager.init();
    // Register Service to Operator
    await contracts.registry.write.registerServiceToOperator([serviceAccount.address], {
      account: operatorAccount.address,
    });

    await started.mineBlock(1); // mine a block to ensure the registration is processed
  }, 60_000);

  test("Lifecycle test", { timeout: 60_000 }, async () => {
    // Alice buys policy for 1_000_000 coverage
    let alice = started.generateAccount("alice");
    let res = await policyManager.buyPolicy(1_000_000, alice.address);

    expect(res).toStrictEqual({
      id: 1,
      insuree: alice.address,
      coverage: 1_000_000,
      premium: 20_000,
      boughtAt: expect.any(Number),
      expiryAt: expect.any(Number),
      claimId: null,
    });

    // simulate time passing
    await started.mineBlock(100);

    // Alice proceeds to claim
    let slashingId = await policyManager.claimPolicy(res.id);

    // manually mine block without moving timestamp so that simulate request slashing returns correct id
    await started.getClient().mine({ blocks: 1 });

    // guardrail approves the slashing request
    await contracts.router.write.guardrailApprove([slashingId, true], { account: guardrailAccount });

    await started.mineBlock(1); // mine a block to ensure the approval is processed

    // proceed to process claim
    let processClaimRes = await policyManager.processClaim(slashingId);

    expect(processClaimRes).toStrictEqual({
      policyDetails: expect.any(Object),
      txHash: expect.any(String),
      payout: 1_000_000,
      payoutAt: expect.any(Number),
    });

    await started.mineBlock(1); // mine a block to ensure the claim is processed

    // check alice balance should have 1_000_000 asset from payout
    let aliceBalance = await underlyingAssetContract.read.balanceOf([alice.address]);
    expect(aliceBalance).toStrictEqual(1_000_000n);
  });

  test("Rewards lifecycle", { timeout: 60_000 }, async () => {
    // Alice buys policy for 80_000_000 coverage
    let alice = started.generateAccount("alice");
    let res = await policyManager.buyPolicy(80_000_000, alice.address);

    expect(res).toStrictEqual({
      id: 1,
      insuree: alice.address,
      coverage: 80_000_000,
      premium: 1_600_000,
      boughtAt: expect.any(Number),
      expiryAt: expect.any(Number),
      claimId: null,
    });

    // calculate rewards
    vi.useFakeTimers();
    vi.advanceTimersByTime(24 * 60 * 60 * 1000); // advance by 24 hour
    let earners = [stakerAccount.address];
    let distributionRewardsRes = await policyManager.calculateRewards(earners);

    expect(distributionRewardsRes).toStrictEqual({
      rewardsToPayout: 2191,
      durationSinceLastUpdate: expect.any(Number),
      totalAccumulatedRewards: 2191,
      rewardsDistribution: {
        token: underlyingAssetContract.address,
        earners: [
          {
            earner: stakerAccount.address,
            reward: "1972",
          },
          {
            earner: operatorAccount.address,
            reward: "219",
          },
        ],
      },
    });

    // mint rewards token to the service account for distribution
    await underlyingAssetContract.write.mint([serviceAccount.address, distributionRewardsRes.rewardsToPayout]);
    await started.mineBlock(1); // mine a block to ensure the mint is processed

    vi.useRealTimers();
    // submit rewards distribution
    await policyManager.submitRewards({
      amountToDistribute: distributionRewardsRes!.rewardsToPayout,
    });

    await started.mineBlock(1); // mine a block to ensure the rewards distribution is processed

    // assert that the rewards contract received the rewards distribution
    let balance = await underlyingAssetContract.read.balanceOf([contracts.rewards.address]);
    expect(balance).toStrictEqual(2191n);

    // staker claims rewards and sends it to recipient address
    const inputFile = `dist/eth-mainnet/${serviceAccount.address}/${underlyingAssetContract.address}/distribution.json`;
    const { stdout } = await execa("satlayer", ["rewards", "proof", stakerAccount.address, "1972", "-f", inputFile], {
      preferLocal: true,
    });
    let proofRes = JSON.parse(stdout.trim());

    let recipientAccount = started.generateAccount("recipient");
    await contracts.rewards.write.claimRewards(
      [
        {
          provider: serviceAccount.address,
          token: proofRes.token,
          amount: proofRes.amount,
          recipient: recipientAccount.address,
          merkleRoot: `0x${proofRes.claim_rewards_proof.root}`,
          proof: proofRes.claim_rewards_proof.proof.map((p: string) => `0x${p}`),
          leafIndex: proofRes.claim_rewards_proof.leaf_index,
          totalLeaves: proofRes.claim_rewards_proof.total_leaves_count,
        },
      ],
      { account: stakerAccount.address },
    );

    await started.mineBlock(1); // mine a block

    // expect that the recipient received the rewards
    let recipientBalance = await underlyingAssetContract.read.balanceOf([recipientAccount.address]);
    expect(recipientBalance).toStrictEqual(1972n);

    // calculate rewards for the 2nd day
    vi.useFakeTimers();
    vi.advanceTimersByTime(2 * 24 * 60 * 60 * 1000); // advance by 48 hour
    let distributionRewardsRes2 = await policyManager.calculateRewards(earners);

    expect(distributionRewardsRes2).toStrictEqual({
      rewardsToPayout: 2191,
      durationSinceLastUpdate: expect.any(Number),
      totalAccumulatedRewards: 4382,
      rewardsDistribution: {
        token: underlyingAssetContract.address,
        earners: [
          {
            earner: stakerAccount.address,
            reward: "3944",
          },
          {
            earner: operatorAccount.address,
            reward: "438",
          },
        ],
      },
    });

    // mint rewards token to the service account for distribution
    await underlyingAssetContract.write.mint([serviceAccount.address, distributionRewardsRes2.rewardsToPayout]);
    await started.mineBlock(1); // mine a block to ensure the mint is processed

    vi.useRealTimers();
    // submit rewards distribution
    await policyManager.submitRewards({
      amountToDistribute: distributionRewardsRes2!.rewardsToPayout,
    });

    // staker claims rewards again
    const { stdout: stdout2 } = await execa(
      "satlayer",
      ["rewards", "proof", stakerAccount.address, "3944", "-f", inputFile],
      { preferLocal: true },
    );
    let proofRes2 = JSON.parse(stdout2.trim());

    await contracts.rewards.write.claimRewards(
      [
        {
          provider: serviceAccount.address,
          token: proofRes2.token,
          amount: proofRes2.amount,
          recipient: recipientAccount.address,
          merkleRoot: `0x${proofRes2.claim_rewards_proof.root}`,
          proof: proofRes2.claim_rewards_proof.proof.map((p: string) => `0x${p}`),
          leafIndex: proofRes2.claim_rewards_proof.leaf_index,
          totalLeaves: proofRes2.claim_rewards_proof.total_leaves_count,
        },
      ],
      { account: stakerAccount.address },
    );
    await started.mineBlock(1); // mine a block

    // expect that the recipient received the rewards
    let recipientBalance2 = await underlyingAssetContract.read.balanceOf([recipientAccount.address]);
    expect(recipientBalance2).toStrictEqual(3944n);
  });
});
