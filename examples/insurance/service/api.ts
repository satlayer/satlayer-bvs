import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { ExecuteMsg as RouterExecuteMsg, QueryMsg as RouterQueryMsg } from "@satlayer/cosmwasm-schema/vault-router";
import { AllAccountsResponse, QueryMsg as VaultBankQueryMsg } from "@satlayer/cosmwasm-schema/vault-bank-tokenized";
import { Coin } from "@cosmjs/proto-signing";
import { ExecuteMsg as RewardsExecuteMsg, RewardsType } from "@satlayer/cosmwasm-schema/rewards";
import { coins } from "@cosmjs/stargate";

interface ApiServiceOptions {
  client: SigningCosmWasmClient;
  vault: string;
  registry: string;
  router: string;
  operator: string;
  service: string;
  rewards: string;
}

/*
 * Api serves as the gateway to the on-chain contract communications
 */
export class Api {
  private readonly client: SigningCosmWasmClient;
  private readonly vault: string;
  private readonly registry: string;
  private readonly router: string;
  private readonly operator: string;
  private readonly service: string;
  private readonly rewards: string;

  constructor({ client, vault, registry, router, operator, service, rewards }: ApiServiceOptions) {
    this.client = client;
    this.vault = vault;
    this.registry = registry;
    this.router = router;
    this.operator = operator;
    this.service = service;
    this.rewards = rewards;
  }

  get Client(): SigningCosmWasmClient {
    return this.client;
  }
  get Vault(): string {
    return this.vault;
  }
  get Registry(): string {
    return this.registry;
  }
  get Router(): string {
    return this.router;
  }
  get Operator(): string {
    return this.operator;
  }
  get Service(): string {
    return this.service;
  }
  get Rewards(): string {
    return this.rewards;
  }

  async queryTotalVaultStakedAmount(): Promise<number> {
    let vaultBalanceMsg: VaultBankQueryMsg = {
      total_assets: {},
    };
    return await this.client.queryContractSmart(this.vault, vaultBalanceMsg);
  }

  async queryStakedAmount({ address }: { address: string }): Promise<bigint> {
    let sharesMsg: VaultBankQueryMsg = {
      shares: {
        staker: address,
      },
    };
    let response = await this.client.queryContractSmart(this.vault, sharesMsg);
    return BigInt(response) || BigInt(0); // Ensure we return a number
  }

  async executeRequestSlashing({
    payoutAmount,
    capacityFactor,
    reason,
  }: {
    payoutAmount: number;
    capacityFactor: number;
    reason: string;
  }): Promise<any> {
    // get total staked
    let totalVaultBalance = await this.queryTotalVaultStakedAmount();

    if (payoutAmount > totalVaultBalance) {
      throw new Error("Payout amount exceeds total vault balance");
    }

    let bips = Math.floor((payoutAmount / totalVaultBalance) * 10000);

    if (bips > capacityFactor * 10000) {
      throw new Error("Bips to slash exceeds max bips");
    }
    if (bips < 1) {
      throw new Error("Bips to slash must be at least 1");
    }

    let timestamp = Math.floor(Date.now() - 1000) * 1_000_000;

    let slashingRequestMsg: RouterExecuteMsg = {
      request_slashing: {
        bips,
        metadata: {
          reason,
        },
        operator: this.operator,
        timestamp: timestamp.toString(),
      },
    };
    return this.client.execute(this.service, this.router, slashingRequestMsg, "auto");
  }

  async querySlashingRequestId({ service, operator }: { service: string; operator: string }): Promise<string> {
    let slashingRequestIdMsg: RouterQueryMsg = {
      slashing_request_id: {
        service,
        operator,
      },
    };
    return await this.client.queryContractSmart(this.router, slashingRequestIdMsg);
  }

  async queryBankBalance({ address, denom }: { address: string; denom: string }): Promise<Coin> {
    return await this.client.getBalance(address, denom);
  }

  async executeLockSlashing(slashingRequestId: string): Promise<any> {
    let lockSlashingMsg: RouterExecuteMsg = {
      lock_slashing: slashingRequestId,
    };
    return this.client.execute(this.service, this.router, lockSlashingMsg, "auto");
  }

  async executeFinalizeSlashing(slashingRequestId: string): Promise<any> {
    let finalizeSlashingMsg: RouterExecuteMsg = {
      finalize_slashing: slashingRequestId,
    };
    return this.client.execute(this.service, this.router, finalizeSlashingMsg, "auto");
  }

  async queryVaultAllAccounts(): Promise<AllAccountsResponse> {
    let vaultAccountsMsg: VaultBankQueryMsg = {
      all_accounts: {},
    };
    return await this.client.queryContractSmart(this.vault, vaultAccountsMsg);
  }

  async executeDistributeRewards({
    token,
    amount,
    merkleRoot,
    rewardsType,
  }: {
    token: string;
    amount: string;
    merkleRoot: string;
    rewardsType: RewardsType;
  }): Promise<any> {
    let distributeRewardsMsg: RewardsExecuteMsg = {
      distribute_rewards: {
        reward_type: rewardsType,
        merkle_root: merkleRoot,
        reward_distribution: {
          amount,
          token,
        },
      },
    };
    if (rewardsType === RewardsType.Bank) {
      let funds = coins(amount, token);
      return this.client.execute(this.service, this.rewards, distributeRewardsMsg, "auto", undefined, funds);
    }
    return this.client.execute(this.service, this.rewards, distributeRewardsMsg, "auto");
  }
}
