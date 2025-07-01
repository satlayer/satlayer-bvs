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
  service: string;
  rewards: string;
}

/*
 * Api serves as the gateway to the on-chain contract communications
 *
 * It provides methods to interact with SatLayer core contracts such as
 * - Vault Bank: for managing staked assets
 * - Router: for handling slashing requests
 * - Rewards: for distributing rewards to stakers
 * - Service: governance contract.
 *
 * It abstracts the complexity of contract interactions and provides a simple interface
 * to perform common operations such as querying balances, and staked collateral amounts.
 */
export class Api {
  readonly client: SigningCosmWasmClient;
  readonly vault: string;
  readonly registry: string;
  readonly router: string;
  readonly operator: string;
  readonly service: string;
  readonly rewards: string;

  constructor({ client, vault, registry, router, service, rewards }: ApiServiceOptions) {
    this.client = client;
    this.vault = vault;
    this.registry = registry;
    this.router = router;
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

  async queryBankBalance({ address, denom }: { address: string; denom: string }): Promise<Coin> {
    return await this.client.getBalance(address, denom);
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
