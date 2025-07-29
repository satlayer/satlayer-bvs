import { Api } from "./api";
import { ExecuteMsg as RegistryExecuteMsg } from "@satlayer/cosmwasm-schema/registry";
import { coins } from "@cosmjs/stargate";
import { AllAccountsResponse } from "@satlayer/cosmwasm-schema/vault-bank-tokenized";
import * as path from "node:path";
import * as fs from "node:fs";
import { execa } from "execa";
import { RewardsType } from "@satlayer/cosmwasm-schema/rewards";

type PolicyId = number;
type ClaimId = string;

/*
 * PolicyManager is an off-chain service
 * that manages insurance policies with automated underwriting and claims verification.
 * It interacts with SatLayer core on-chain contracts to handle the entire policy lifecycle.
 *
 * Key responsibilities:
 * - Policy lifecycle management (buying policy, claims processing, and payouts)
 * - Rewards calculation and distribution to earners (operators and stakers)
 *
 * Assumptions for this example:
 * - All capital is provided by a single Operator and a single Vault
 * - 80% of the vault balance can be used for policy issuance and claims
 * - Only one operator is registered to the service
 * - The operator manages a single vault
 * - Policies are valid for 1 year
 * - Premiums are fixed at 2% of coverage amount
 * - Policies claims payout 100% of the coverage amount
 * - Policies premium is paid upfront for the entire policy duration (not included in this example)
 * - Rewards are distributed proportionally to staked amounts
 * - Rewards are 50% of the total premiums collected from active policies
 * - Operators receive 10% of rewards, stakers receive 90%
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

  // stores the rewards history for distribution
  private rewardsHistory: DistributionHistory = {
    lastUpdated: Date.now(),
    rewards: {
      token: "ustake",
      earners: [],
    },
  };

  // 80% of the vault balance can be used to issue policies
  private readonly CAPACITY_FACTOR = 0.8;

  private readonly POLICY_DURATION = 365 * 24 * 60 * 60 * 1000; // 1 year

  private currentPolicyId: number = 0;

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

  /**
   * Generates the next unique policy ID.
   * @private
   */
  private getNextPolicyId(): number {
    this.currentPolicyId += 1;
    return this.currentPolicyId;
  }

  /**
   * Calculates the premium for a given coverage amount for the entire policy duration.
   *
   * Uses a simple calculation of 2% of the coverage amount.
   * In a real-world scenario,
   * this would incorporate risk profiles, policy duration, and other factors.
   *
   * @param coverage The amount of coverage for which to calculate the premium
   * @returns The calculated premium amount
   */
  public getPremium(coverage: number): number {
    let basePremiumFactor = 0.02; // 2% of coverage

    // * insert additional logic here for risk assessment, etc. *

    return coverage * basePremiumFactor;
  }

  /**
   * Buy a new insurance policy for the given coverage amount.
   *
   * This function validates if the insuree is eligible to buy a policy and registers the coverage amount.
   * It checks the vault balance to ensure there's sufficient capacity for the new policy.
   *
   * @param coverage The amount of coverage to buy
   * @param insuree The address of the insuree as a unique identifier and address for payout
   * @returns PolicyDetails
   */
  public async buyPolicy(coverage: number, insuree: string): Promise<PolicyDetails> {
    // Validate insuree doesn't already have a policy
    if (this.insureeMap.has(insuree)) {
      throw new Error("Insuree already has a policy");
    }

    // Calculate premium
    const premium = this.getPremium(coverage);

    // Check if the total insured amount would exceed total vault balance capacity
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
   * It will perform the necessary checks to validate the claim.
   * If the claim is verified, it will trigger a slashing request to the vault to initiate and lock the payout immediately.
   * The amount of payout is 100% of the coverage amount.
   *
   * funds flow: vault --lockSlashing--> vault-router
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

    let isClaimedVerified = true; // assume the claim is verified

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
   * Process the claim for the given claimId and finalize the payout to the insuree.
   *
   * This function finalizes the slashing request and transfers the payout amount to the insuree.
   * It also updates the internal state to reflect the completed claim and removes the policy from active policies.
   *
   * Funds flow: vault-router --finalizeSlashing--> service (policy manager) --transfer--> insuree
   *
   * @param claimId The ID of the claim to process
   * @returns PolicyPayout Details of the processed payout
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

    // Finalizes the slashing request
    await this.api.executeFinalizeSlashing(claimId);

    // Proceed to payout slashing amount to the insuree
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
   * Calculates the rewards to be distributed to the Earners (Operators and Stakers).
   *
   * Rewards are calculated based on the time elapsed since the last distribution.
   * The total reward amount is 50% of the premium collected from active policies,
   * prorated for the time period since the last distribution.
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

    // convert to distribution.json file format
    let newDistributionFileData: DistributionRewards = {
      token: "ustake",
      earners: Array.from(accumulatedEarnerRewards.entries()).map(([earner, reward]) => ({
        earner: earner,
        reward: reward.toString(),
      })),
    };

    // write the new distribution.json into the dist folder
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

  /**
   * Submits the rewards to the rewards contract.
   *
   * This function creates a Merkle tree from the distribution.json file using the satlayer CLI tool,
   * then submits the rewards to the rewards contract.
   *
   * It assumes the distribution.json file is already generated by calling `calculateRewards()`
   * and contains the rewards to be distributed.
   * It also assumes that the service has the necessary funds and allowance to transfer the rewards amount to the rewards contract.
   *
   * @param amountToDistribute
   */
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

  /**
   * Creates a Merkle tree from the distribution.json file using the satlayer CLI tool.
   *
   * It uses the `satlayer rewards create` command to generate the Merkle root from the distribution.json file.
   *
   * @param inputFile The path to the distribution.json file
   * @returns The Merkle root hash as a string
   */
  private async createMerkleTree(inputFile: string) {
    const { stdout } = await execa("satlayer", ["rewards", "create", "-f", inputFile], { preferLocal: true });

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
