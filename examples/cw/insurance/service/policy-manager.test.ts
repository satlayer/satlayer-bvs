import { beforeEach, describe, expect, test, vi } from "vitest";
import { PolicyManager } from "./policy-manager";
import { Api } from "./api";
import { CosmWasmContainer, CosmWasmContracts, StartedCosmWasmContainer } from "@satlayer/testcontainers";
import { AccountData, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { stringToPath } from "@cosmjs/crypto";
import { ExecuteMsg as VaultBankExecuteMsg } from "@satlayer/cosmwasm-schema/vault-bank";
import { ExecuteMsg as RegistryExecuteMsg } from "@satlayer/cosmwasm-schema/registry";
import { coins } from "@cosmjs/stargate";
import { sleep } from "@cosmjs/utils";
import { ExecuteMsg as GuardrailExecuteMsg } from "@satlayer/cosmwasm-schema/guardrail";
import { execa } from "execa";
import { ExecuteMsg as RewardsExecuteMsg, RewardsType } from "@satlayer/cosmwasm-schema/rewards";

let policyManager: PolicyManager;
let api: Api;
let started: StartedCosmWasmContainer;
let contracts: CosmWasmContracts;
let wallet: DirectSecp256k1HdWallet;

let vaultAddress: string;
let serviceAccount: AccountData;
let operatorAccount: AccountData;
let ownerAccount: AccountData;
let stakerAccount: AccountData;
let rewardsAccount: AccountData;

describe("PolicyManager", () => {
  beforeEach(async () => {
    // Set up CosmWasmContainer with CosmWasmContracts bootstrapped
    started = await new CosmWasmContainer().start();
    contracts = await CosmWasmContracts.bootstrap(started);

    // A wallet with 4 accounts operator, staker and service
    wallet = await DirectSecp256k1HdWallet.generate(12, {
      prefix: "wasm",
      hdPaths: [stringToPath("m/0"), stringToPath("m/1"), stringToPath("m/2")],
    });
    const [operator, staker, service] = await wallet.getAccounts();

    // get owner from container
    const [owner] = await started.wallet.getAccounts();

    serviceAccount = service;
    operatorAccount = operator;
    ownerAccount = owner;
    stakerAccount = staker;

    // Fund all 4 accounts with some tokens
    await started.fund("1000000000ustake", owner.address, operator.address, staker.address, service.address);

    // Init vault bank tokenized for operator
    vaultAddress = await contracts.initVaultBankTokenized(operator.address, "ustake", "satUstake");

    // Initialize Client
    let clientSigner = await started.newSigner(wallet);
    api = new Api({
      client: clientSigner,
      vault: vaultAddress,
      registry: contracts.registry.address,
      router: contracts.router.address,
      operator: operator.address,
      service: service.address,
      rewards: contracts.rewards.address,
    });

    // Operator register into Registry
    let registerOperatorMsg: RegistryExecuteMsg = {
      register_as_operator: {
        metadata: {
          name: "Operator",
          uri: "https://operator.com",
        },
      },
    };
    await clientSigner.execute(operator.address, contracts.registry.address, registerOperatorMsg, "auto");

    // Staker stake into vault bank
    let stakeMsg: VaultBankExecuteMsg = {
      deposit_for: {
        amount: "100000000", // stake 100_000_000 ustake
        recipient: staker.address,
      },
    };
    await clientSigner.execute(staker.address, vaultAddress, stakeMsg, "auto", undefined, coins(100_000_000, "ustake"));

    // Initialize PolicyManager
    policyManager = new PolicyManager(api, serviceAccount.address, operator.address);
    await policyManager.init();

    // Register Service to Operator
    let registerServiceToOperatorMsg: RegistryExecuteMsg = {
      register_service_to_operator: {
        service: serviceAccount.address,
      },
    };
    await clientSigner.execute(operator.address, contracts.registry.address, registerServiceToOperatorMsg, "auto");
  }, 60_000);

  test("Lifecycle test", { timeout: 60_000 }, async () => {
    await sleep(500);
    // Alice buys policy for 1_000_000 coverage
    let alice = await started.generateAccount("alice");
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

    await sleep(1000);

    // Alice proceeds to claim
    let claimRes = await policyManager.claimPolicy(res.id);
    // expect claimRes to be slashing request id with 64 hex chars (32 bytes)
    expect(claimRes).toMatch(/^[0-9a-fA-F]{64}$/);

    // propose to guardrail
    let proposeMsg: GuardrailExecuteMsg = {
      propose: {
        reason: "Payout",
        slashing_request_id: claimRes,
      },
    };
    await started.client.execute(ownerAccount.address, contracts.guardrail.address, proposeMsg, "auto");

    await sleep(1000);

    // proceed to process claim
    let processClaimRes = await policyManager.processClaim(claimRes);

    expect(processClaimRes).toStrictEqual({
      policyDetails: expect.any(Object),
      txHash: expect.any(String),
      payout: 1_000_000,
      payoutAt: expect.any(Number),
    });

    // check alice balance should have 1_000_000 ustake from payout
    let aliceBalance = await api.queryBankBalance({ address: alice.address, denom: "ustake" });
    expect(aliceBalance).toStrictEqual({
      amount: "1000000",
      denom: "ustake",
    });
  });

  test("Rewards lifecycle", { timeout: 60_000 }, async () => {
    await sleep(500);
    // Alice buys policy for 80_000_000 coverage
    let alice = await started.generateAccount("alice");
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

    await sleep(1000);

    // calculate rewards
    vi.useFakeTimers();
    vi.advanceTimersByTime(24 * 60 * 60 * 1000); // advance by 24 hour
    let distributionRewardsRes = await policyManager.calculateRewards();

    expect(distributionRewardsRes).toStrictEqual({
      rewardsToPayout: 2191,
      durationSinceLastUpdate: expect.any(Number),
      totalAccumulatedRewards: 2191,
      rewardsDistribution: {
        token: "ustake",
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

    vi.useRealTimers();
    // submit rewards distribution
    await policyManager.submitRewards({
      amountToDistribute: distributionRewardsRes!.rewardsToPayout,
    });

    // assert that the rewards contract received the rewards distribution
    let balance = await api.queryBankBalance({
      address: contracts.rewards.address,
      denom: "ustake",
    });
    expect(balance).toStrictEqual({
      amount: "2191",
      denom: "ustake",
    });

    // staker claims rewards
    const inputFile = `dist/bbn-test-5/${serviceAccount.address}/ustake/distribution.json`;
    const { stdout } = await execa("satlayer", ["rewards", "proof", stakerAccount.address, "1972", "-f", inputFile], {
      preferLocal: true,
    });
    let proofRes = JSON.parse(stdout.trim());

    let recipientAccount = await started.generateAccount("recipient");
    let claimRewardsMsg: RewardsExecuteMsg = {
      claim_rewards: {
        token: proofRes.token,
        amount: proofRes.amount,
        recipient: recipientAccount.address,
        reward_type: RewardsType.Bank,
        service: serviceAccount.address,
        claim_rewards_proof: {
          leaf_index: proofRes.claim_rewards_proof.leaf_index,
          proof: proofRes.claim_rewards_proof.proof,
          root: proofRes.claim_rewards_proof.root,
          total_leaves_count: proofRes.claim_rewards_proof.total_leaves_count,
        },
      },
    };
    let clientSigner = await started.newSigner(wallet);
    await clientSigner.execute(stakerAccount.address, contracts.rewards.address, claimRewardsMsg, "auto");

    // expect that the recipient received the rewards
    let recipientBalance = await api.queryBankBalance({
      address: recipientAccount.address,
      denom: "ustake",
    });
    expect(recipientBalance).toStrictEqual({
      amount: "1972",
      denom: "ustake",
    });

    // calculate rewards for the 2nd day
    vi.useFakeTimers();
    vi.advanceTimersByTime(2 * 24 * 60 * 60 * 1000); // advance by 48 hour
    let distributionRewardsRes2 = await policyManager.calculateRewards();

    expect(distributionRewardsRes2).toStrictEqual({
      rewardsToPayout: 2191,
      durationSinceLastUpdate: expect.any(Number),
      totalAccumulatedRewards: 4382,
      rewardsDistribution: {
        token: "ustake",
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

    let claimRewardsMsg2: RewardsExecuteMsg = {
      claim_rewards: {
        token: proofRes2.token,
        amount: proofRes2.amount,
        recipient: recipientAccount.address,
        reward_type: RewardsType.Bank,
        service: serviceAccount.address,
        claim_rewards_proof: {
          leaf_index: proofRes2.claim_rewards_proof.leaf_index,
          proof: proofRes2.claim_rewards_proof.proof,
          root: proofRes2.claim_rewards_proof.root,
          total_leaves_count: proofRes2.claim_rewards_proof.total_leaves_count,
        },
      },
    };
    await clientSigner.execute(stakerAccount.address, contracts.rewards.address, claimRewardsMsg2, "auto");

    // expect that the recipient received the rewards
    let recipientBalance2 = await api.queryBankBalance({
      address: recipientAccount.address,
      denom: "ustake",
    });
    expect(recipientBalance2).toStrictEqual({
      amount: "3944",
      denom: "ustake",
    });
  });
});
