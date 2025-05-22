import { Api } from "./api";
import { ExecuteMsg as RegistryExecuteMsg } from "@satlayer/cosmwasm-schema/registry";
import { coins } from "@cosmjs/stargate";
import { sleep } from "@cosmjs/utils";

type PolicyId = number;

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
  private totalInsured: number = 0;
  private policyMap: Map<PolicyId, PolicyDetails> = new Map();
  private insureeMap: Map<string, PolicyId> = new Map();

  // stores the slashing request ID and the policy ID map
  private slashingMap: Map<string, PolicyId> = new Map();
  private payoutList: Array<PolicyPayout> = new Array<PolicyPayout>();

  private readonly CAPACITY_FACTOR = 0.8; // 80% of the vault balance can be insured

  private currentPolicyId: number = 0;

  constructor(
    private readonly api: Api,
    private readonly service: string,
    private readonly operator: string,
  ) {}

  // Initialize the PolicyManager by:
  // - Registering the service to the SatLayer Registry
  // - Enable slashing
  // - Register the operator
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
          resolution_window: 1, // immediate slashing
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
  }

  private getNextPolicyId(): number {
    this.currentPolicyId += 1;
    return this.currentPolicyId;
  }

  /// Calculates the premium for a given coverage and duration.
  public getPremium(coverage: number): number {
    let basePremiumFactor = 0.005; // 0.5% of coverage
    // TODO: add more complex premium calculation logic here?
    return coverage * basePremiumFactor;
  }

  /**
   * Buy a new insurance policy for the given coverage amount.
   * @param coverage The amount of coverage to buy
   * @param insuree The name of the insuree
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
    let totalVaultBalance = await this.api.getStakedAmount();
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
      expiryAt: Date.now() + 365 * 24 * 60 * 60 * 1000, // 1 year
    };
    this.policyMap.set(newPolicy.id, newPolicy);
    this.insureeMap.set(insuree, newPolicy.id);

    return newPolicy;
  }

  /*
   * Claim a policy for the given policy ID.
   * @param policyId The ID of the policy to claim
   */
  public async claimPolicy(policyId: number): Promise<string> {
    // Validate policy exists
    if (!this.policyMap.has(policyId)) {
      throw new Error("Policy does not exist");
    }

    // Validate policy is not expired
    let policy = this.policyMap.get(policyId);
    if (policy.expiryAt < Date.now()) {
      throw new Error("Policy has expired");
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

    let slashingRequestId = await this.api.getSlashingRequestId({ service: this.service, operator: this.operator });

    // sleep to pass the resolution_window
    await sleep(1000);

    // lock the slashing
    await this.api.executeLockSlashing(slashingRequestId);

    this.slashingMap.set(slashingRequestId, policyId);

    return slashingRequestId;
  }

  public async processClaim(slashingRequestId: string): Promise<PolicyPayout> {
    // Validate slashing request ID exists
    if (!this.slashingMap.has(slashingRequestId)) {
      throw new Error("Slashing request ID does not exist");
    }

    let policyId = this.slashingMap.get(slashingRequestId);

    // Validate policy exists
    if (!this.policyMap.has(policyId)) {
      throw new Error("Policy does not exist");
    }
    let policy = this.policyMap.get(policyId);

    // finalizes the slashing request
    await this.api.executeFinalizeSlashing(slashingRequestId);

    // proceeed to payout slashing amount to the insuree (assumes that the slash amt has been transferred)
    let payoutAmount = policy.coverage;
    let res = await this.api.Client.sendTokens(this.service, policy.insuree, coins(payoutAmount, "ustake"), "auto");

    // update state
    this.totalInsured -= payoutAmount;
    this.policyMap.delete(policyId);
    this.insureeMap.delete(policy.insuree);
    this.slashingMap.delete(slashingRequestId);

    let payout: PolicyPayout = {
      policyDetails: policy,
      txHash: res.transactionHash,
      payout: payoutAmount,
      payoutAt: Date.now(),
    };
    this.payoutList.push(payout);

    return payout;
  }
}

export interface PolicyDetails {
  /// The policy unique ID
  id: number;
  // The insuree address
  insuree: string;
  // The policy coverage amount
  coverage: number;
  // The policy premium amount
  premium: number;
  // The timestamp in ms when the policy was bought
  boughtAt: number;
  // The timestamp in ms when the policy expires
  expiryAt: number;
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
