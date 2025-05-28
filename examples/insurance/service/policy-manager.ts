import { Api } from "./api";
import { ExecuteMsg as RegistryExecuteMsg } from "@satlayer/cosmwasm-schema/registry";
import { coins } from "@cosmjs/stargate";
import { sleep } from "@cosmjs/utils";
import { AllAccountsResponse } from "@satlayer/cosmwasm-schema/vault-bank-tokenized";
import * as path from "node:path";
import * as fs from "node:fs";
import { execa } from "execa";
import { resolve } from "path";
import { RewardsType } from "@satlayer/cosmwasm-schema/rewards";

type PolicyId = number;
type ClaimId = string;

/*
 * PolicyManager manages 1 policy that has automated underwriting + automated claims verification.
 * It is an off-chain service runs by the Service that interacts with the on-chain contract.
 *
 * The PolicyManager is responsible for:
 * - Managing the policy lifecycle (underwriting, claims verification, etc.)
 * - Interacting with the on-chain contract to perform actions such as underwriting, claims verification, etc.
 *
 * This example assumes that all the capital is provided by the Operator.
 */
export class PolicyManager {
  /* State Storage */
  // stores the total insured amount
  private totalInsured: number = 0;
  // stores the map of Policy ID to policy details
  private policyMap: Map<PolicyId, PolicyDetails> = new Map();
  // stores the map of insuree address to Policy ID
  private insureeMap: Map<string, PolicyId> = new Map();

  // stores the ClaimId and the Policy ID map
  private claimMap: Map<ClaimId, PolicyId> = new Map();
  // stores the payout history as a list
  private payoutHistory: Array<PolicyPayout> = new Array<PolicyPayout>();

  private rewardsHistory: DistributionHistory = {
    lastUpdated: Date.now(),
    rewards: {
      token: "ustake",
      earners: [],
    },
  };

  // 80% of the vault balance can be used to issue policies
  private readonly CAPACITY_FACTOR = 0.8;

  private currentPolicyId: number = 0;

  private readonly POLICY_DURATION = 365 * 24 * 60 * 60 * 1000; // 1 year

  constructor(
    private readonly api: Api,
    private readonly service: string,
    private readonly operator: string,
  ) {}

  /**
   * Initialize the PolicyManager by:
   *  - Registering the service to the SatLayer Registry
   *  - Enable slashing
   *  - Register the operator
   */
  public async init() {
    // register service to SatLayer Registry
    let registerAsServiceMsg: RegistryExecuteMsg = {
      register_as_service: {
        metadata: {
          name: "The Sure Coverage",
          uri: "https://the-sure-coverage.com",
        },
      },
    };
    await this.api.Client.execute(this.service, this.api.Registry, registerAsServiceMsg, "auto");

    // enable slashing
    let enableSlashingMsg: RegistryExecuteMsg = {
      enable_slashing: {
        slashing_parameters: {
          destination: this.service, // slash amt goes to the service addr to be paid out to the insuree
          max_slashing_bips: this.CAPACITY_FACTOR * 10000, // 80% of the vault balance can be slashed
          resolution_window: 0, // immediate slashing
        },
      },
    };
    await this.api.Client.execute(this.service, this.api.Registry, enableSlashingMsg, "auto");

    // register operator to the service
    let registerOperatorMsg: RegistryExecuteMsg = {
      register_operator_to_service: {
        operator: this.operator,
      },
    };
    await this.api.Client.execute(this.service, this.api.Registry, registerOperatorMsg, "auto");

    // init rewards history
    this.rewardsHistory = {
      lastUpdated: Date.now(),
      rewards: {
        token: "ustake",
        earners: [],
      },
    };
  }

  private getNextPolicyId(): number {
    this.currentPolicyId += 1;
    return this.currentPolicyId;
  }

  /**
   * Calculates the premium for a given coverage amount for the entire policy duration.
   *
   * Logic can be more complex based on the risk profile, duration, etc.
   * @param coverage
   */
  public getPremium(coverage: number): number {
    let basePremiumFactor = 0.02; // 2% of coverage
    // TODO: add more complex premium calculation logic here?
    return coverage * basePremiumFactor;
  }

  /**
   * Buy a new insurance policy for the given coverage amount.
   *
   * This function mostly checks if insuree is able to buy the policy and register the coverage amount.
   * There is no on-chain interaction, other than checking vault balance to determine capacity.
   *
   * This fn assumes:
   *  - 80% of the vault balance can be used to issue policies and paid out in case of any claims.
   *  - Only 1 operator is registered to the service.
   *  - The operator only has 1 vault.
   *  - The payout denom is the same as the coverage denom.
   *  - The policy is valid for 1 year.
   * @param coverage The amount of coverage to buy
   * @param insuree The address of the insuree for payout
   * @returns PolicyDetails
   */
  public async buyPolicy(coverage: number, insuree: string): Promise<PolicyDetails> {
    // Validate insuree doesn't already have a policy
    if (this.insureeMap.has(insuree)) {
      throw new Error("Insuree already has a policy");
    }

    // Calculate premium
    const premium = this.getPremium(coverage);

    // Check if total insured amount would exceed total vault balance capacity
    let totalVaultBalance = await this.api.queryTotalVaultStakedAmount();
    if (this.totalInsured + coverage > totalVaultBalance * this.CAPACITY_FACTOR) {
      throw new Error("Coverage amount would exceed total insurance capacity");
    }

    // * add additional logic here for underwriting, etc. *

    // SUCCESS

    // Update local state
    this.totalInsured += coverage;
    let newPolicy: PolicyDetails = {
      id: this.getNextPolicyId(),
      insuree: insuree,
      coverage: coverage,
      premium: premium,
      boughtAt: Date.now(),
      expiryAt: Date.now() + this.POLICY_DURATION, // 1 year
      claimId: null,
    };
    this.policyMap.set(newPolicy.id, newPolicy);
    this.insureeMap.set(insuree, newPolicy.id);

    return newPolicy;
  }

  /**
   * Claim a policy for the given policy ID.
   *
   * This function is used in the case of a claim from the insuree.
   * It will perform necessary checks to validate the claim.
   * If the claim is verified, it will trigger a slashing request to the vault to initiate and lock the payout immediately.
   * The amount of payout is 100% of the coverage amount.
   *
   * Once claimed, the policy will be nulled and the insuree will not be able to claim again.
   * @param policyId The ID of the policy to claim
   * @return ClaimId The ID of the claim, which is the same as the slashing request ID
   */
  public async claimPolicy(policyId: number): Promise<ClaimId> {
    // Validate policy exists
    if (!this.policyMap.has(policyId)) {
      throw new Error("Policy does not exist");
    }

    // Validate policy is not expired
    let policy = this.policyMap.get(policyId)!;
    if (policy.expiryAt < Date.now()) {
      throw new Error("Policy has expired");
    }

    // check if the policy is already claimed
    if (policy.claimId != null) {
      throw new Error("Policy has already been claimed");
    }

    // * add extra logic here for claims verification, etc. *

    let isClaimedVerified = true; // assume claim is verified

    if (!isClaimedVerified) {
      throw new Error("Claim is rejected");
    }

    // payout 100% of the coverage amount
    let payoutAmount = policy.coverage;

    // trigger slashing of the vault to payout to the insuree
    await this.api.executeRequestSlashing({
      payoutAmount,
      capacityFactor: this.CAPACITY_FACTOR,
      reason: `Payout for policy ${policyId}`,
    });

    let slashingRequestId = await this.api.querySlashingRequestId({ service: this.service, operator: this.operator });

    // sleep to pass the resolution_window
    await sleep(1000);

    // lock the slashing
    await this.api.executeLockSlashing(slashingRequestId);

    // update state
    this.claimMap.set(slashingRequestId, policyId);
    this.policyMap.set(policyId, {
      ...policy,
      claimId: slashingRequestId, // set the claim ID to the slashing request ID
    });

    return slashingRequestId;
  }

  /**
   * Process the claim for the given claimId
   *
   * This function is used to trigger the payout to the insuree after the guardrail has voted on it.
   * @param claimId
   */
  public async processClaim(claimId: ClaimId): Promise<PolicyPayout> {
    // Validate slashing request ID exists
    if (!this.claimMap.has(claimId)) {
      throw new Error("Slashing request ID does not exist");
    }

    let policyId = this.claimMap.get(claimId)!;

    // Validate policy exists
    if (!this.policyMap.has(policyId)) {
      throw new Error("Policy does not exist");
    }
    let policy = this.policyMap.get(policyId)!;

    // finalizes the slashing request
    await this.api.executeFinalizeSlashing(claimId);

    // proceeed to payout slashing amount to the insuree (assumes that the slash amt has been transferred)
    let payoutAmount = policy.coverage;
    let res = await this.api.Client.sendTokens(this.service, policy.insuree, coins(payoutAmount, "ustake"), "auto");

    // update state
    this.totalInsured -= payoutAmount;
    this.insureeMap.delete(policy.insuree);

    let payout: PolicyPayout = {
      policyDetails: policy,
      txHash: res.transactionHash,
      payout: payoutAmount,
      payoutAt: Date.now(),
    };
    this.payoutHistory.push(payout);

    return payout;
  }

  /**
   * Calculates the rewards to be distributed to the Operators and stakers.
   *
   * Rewards will be calculated daily and distributed to the rewards contract.
   * Rewards will be 50% of the total premium collected from the policies.
   * This fn assumes:
   *  - Rewards distributed are proportional to the amount staked
   *  - Rewards are distributed in the same denom as the vault denom and coverage denom
   *  - Operator get 10% of the rewards from the stakers
   */
  public async calculateRewards(): Promise<RewardsCalculationResult> {
    let now = Date.now();

    // assumes a simple premium calculation of 2% of the coverage amount
    let totalPremium = this.getPremium(this.totalInsured);

    // get the duration since last rewards calculation
    let durationSinceLastUpdate = now - this.rewardsHistory.lastUpdated!;

    // calculate the rewards amount based on the total premium collected and pro-rate it based on the duration since last update
    let rewardsAmount = Math.floor(totalPremium * 0.5 * (durationSinceLastUpdate / this.POLICY_DURATION)); // 50% of the total premium collected

    // 10% of the rewards go to the operator. Assumes only 1 operator.
    let operatorRewards = Math.floor(rewardsAmount * 0.1);

    // 90% of the rewards go to the stakers
    let stakerRewards = Math.floor(rewardsAmount - operatorRewards);

    // get all vault bank stakers address
    let allAccountsRes: AllAccountsResponse = await this.api.queryVaultAllAccounts();

    // get each staker's balance and calculate total staked amount
    let stakerBalanceMap = new Map<string, bigint>();
    let totalStaked = BigInt(0);
    for (let account of allAccountsRes.accounts) {
      // get each staker receipt token balance ( = amount staked )
      let stakerShare = await this.api.queryStakedAmount({ address: account });
      stakerBalanceMap.set(account, stakerShare);
      totalStaked += stakerShare;
    }

    if (totalStaked === BigInt(0)) {
      throw new Error("No stakers found or total staked amount is zero");
    }

    // calculate rewards for each staker proportional to their stake/balance
    let stakerRewardsMap = new Map<string, bigint>();
    for (let account of allAccountsRes.accounts) {
      let balance = stakerBalanceMap.get(account) || BigInt(0);
      let stakerRewardsAmount = (balance / totalStaked) * BigInt(stakerRewards);
      stakerRewardsMap.set(account, stakerRewardsAmount);
    }

    // transform earner rewards to a map of the earner address to reward amount
    let accumulatedEarnerRewards = new Map<string, bigint>();
    let totalAccumulatedRewards = BigInt(0);
    for (let earner of this.rewardsHistory.rewards.earners) {
      let earnerAddress = earner.earner;
      let rewardAmount = BigInt(earner.reward);
      let accumulatedReward = accumulatedEarnerRewards.get(earnerAddress) || BigInt(0);
      accumulatedEarnerRewards.set(earnerAddress, accumulatedReward + rewardAmount);
      totalAccumulatedRewards += accumulatedReward + rewardAmount;
    }

    // add the rewards to the staker's total accumulated rewards
    for (let [earner, reward] of stakerRewardsMap) {
      let accumulatedReward = accumulatedEarnerRewards.get(earner) || BigInt(0);
      accumulatedEarnerRewards.set(earner, accumulatedReward + BigInt(reward));
      totalAccumulatedRewards += BigInt(reward);
    }

    // add the rewards to the operator's total accumulated rewards
    let operatorAccumulatedReward = accumulatedEarnerRewards.get(this.operator) || BigInt(0);
    accumulatedEarnerRewards.set(this.operator, operatorAccumulatedReward + BigInt(operatorRewards));
    totalAccumulatedRewards += BigInt(operatorRewards);

    // convert to distribuiton.json file format
    let newDistributionFileData: DistributionRewards = {
      token: "ustake",
      earners: Array.from(accumulatedEarnerRewards.entries()).map(([earner, reward]) => ({
        earner: earner,
        reward: reward.toString(),
      })),
    };

    // write the new distribution.json into dist folder
    const distDir = path.resolve(process.cwd(), `dist/bbn-test-5/${this.api.Service}/ustake`);
    fs.mkdirSync(distDir, { recursive: true });

    const filePath = path.join(distDir, "distribution.json");
    fs.writeFileSync(filePath, JSON.stringify(newDistributionFileData, null, 2), "utf8");

    // update state
    this.rewardsHistory = {
      lastUpdated: now,
      rewards: newDistributionFileData,
    };

    return {
      rewardsToPayout: rewardsAmount,
      durationSinceLastUpdate: durationSinceLastUpdate,
      totalAccumulatedRewards: Number(totalAccumulatedRewards),
      rewardsDistribution: newDistributionFileData,
    };
  }

  public async submitRewards({ amountToDistribute }: { amountToDistribute: number }): Promise<string> {
    // create the merkle tree and generate the root hash to submit to the rewards contract
    let merkleTreeRoot = await this.createMerkleTree(`dist/bbn-test-5/${this.api.Service}/ustake/distribution.json`);

    // distribute the rewards to the rewards contract
    await this.api.executeDistributeRewards({
      token: "ustake",
      amount: amountToDistribute.toString(),
      merkleRoot: merkleTreeRoot,
      rewardsType: RewardsType.Bank,
    });

    return merkleTreeRoot;
  }

  // returns the merkle root
  private async createMerkleTree(inputFile: string) {
    const binPath = resolve(process.cwd(), "node_modules", ".bin", "satlayer");
    const { stdout } = await execa(binPath, ["rewards", "create", "-f", inputFile]);

    // Parse the Merkle root line
    const match = stdout.match(/Merkle root:\s*([0-9a-fA-F]{64})/);
    if (!match) {
      throw new Error("Failed to parse Merkle root from output");
    }

    return match[1];
  }
}

export interface PolicyDetails {
  /// The policy unique ID
  id: number;
  // The insuree address
  insuree: string;
  // The policy coverage amount
  coverage: number;
  // The policy premium amount to be paid for the entire duration
  premium: number;
  // The timestamp in ms when the policy was bought
  boughtAt: number;
  // The timestamp in ms when the policy expires
  expiryAt: number;
  // Stores the claim ID if the policy is claimed
  claimId: string | null;
}

export interface PolicyPayout {
  // The policy details
  policyDetails: PolicyDetails;
  // The transaction hash of the payout
  txHash: string;
  // The payout amount
  payout: number;
  // The timestamp in ms when the payout was made
  payoutAt: number;
}

export interface DistributionRewards {
  // The token to be distributed
  token: string;
  // The list of earners with their rewards
  earners: Array<{
    earner: string; // address of the earner
    reward: string; // amount of reward in string format
  }>;
}

export interface RewardsCalculationResult {
  rewardsToPayout: number; // total rewards for this calculation
  durationSinceLastUpdate: number; // duration since last rewards calculation in ms
  totalAccumulatedRewards: number; // total accumulated rewards for all earners from the start
  rewardsDistribution: DistributionRewards; // the rewards to be distributed
}

export interface DistributionHistory {
  lastUpdated?: number; // timestamp in ms when the distribution was last updated
  rewards: DistributionRewards; // the rewards to be distributed
}
