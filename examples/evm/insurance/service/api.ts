import { StartedAnvilContainer, SuperTestClient } from "@satlayer/testcontainers";
import { getContract, GetContractReturnType } from "viem";

import vaultAbi from "@satlayer/contracts/SLAYVaultV2.sol/SLAYVaultV2.json";
import registryAbi from "@satlayer/contracts/SLAYRegistryV2.sol/SLAYRegistryV2.json";
import routerAbi from "@satlayer/contracts/SLAYRouterV2.sol/SLAYRouterV2.json";
import rewardsAbi from "@satlayer/contracts/SLAYRewardsV2.sol/SLAYRewardsV2.json";

interface ApiServiceOptions {
  startedContainer: StartedAnvilContainer;
  vault: `0x${string}`;
  registry: `0x${string}`;
  router: `0x${string}`;
  operator: `0x${string}`;
  service: `0x${string}`;
  rewards: `0x${string}`;
}

/*
 * Api serves as the gateway to the on-chain contract communications
 *
 * It provides methods to interact with SatLayer EVM contracts such as
 * - SLAYVault: for managing staked assets
 * - SLAYRouter: for handling slashing requests
 * - SLAYRewards: for distributing rewards to stakers
 *
 * It abstracts the complexity of contract interactions and provides a simple interface
 * to perform common operations such as querying balances, executing slashing requests,
 * and distributing rewards.
 */
export class Api {
  private readonly client: SuperTestClient;
  private readonly vault: GetContractReturnType<typeof vaultAbi.abi, SuperTestClient, `0x${string}`>;
  private readonly registry: GetContractReturnType<typeof registryAbi.abi, SuperTestClient, `0x${string}`>;
  private readonly router: GetContractReturnType<typeof routerAbi.abi, SuperTestClient, `0x${string}`>;
  private readonly operator: string;
  private readonly service: string;
  private readonly rewards: GetContractReturnType<typeof rewardsAbi.abi, SuperTestClient, `0x${string}`>;

  constructor({ startedContainer, vault, registry, router, operator, service, rewards }: ApiServiceOptions) {
    this.client = startedContainer.getClient();

    this.vault = getContract({
      address: vault,
      abi: vaultAbi.abi,
      client: this.client,
    });
    this.registry = getContract({
      address: registry,
      abi: registryAbi.abi,
      client: this.client,
    });
    this.router = getContract({
      address: router,
      abi: routerAbi.abi,
      client: this.client,
    });
    this.rewards = getContract({
      address: rewards,
      abi: rewardsAbi.abi,
      client: this.client,
    });
    this.operator = operator;
    this.service = service;
  }

  get Client(): SuperTestClient {
    return this.client;
  }
  get VaultContract() {
    return this.vault;
  }
  get RegistryContract() {
    return this.registry;
  }
  get RouterContract() {
    return this.router;
  }
  get Operator(): string {
    return this.operator;
  }
  get Service(): string {
    return this.service;
  }
  get RewardsContract() {
    return this.rewards;
  }

  async queryTotalVaultStakedAmount(): Promise<bigint> {
    return (await this.vault.read.totalAssets()) as Promise<bigint>;
  }

  async queryStakedAmount({ address }: { address: string }): Promise<bigint> {
    return (await this.vault.read.balanceOf([address])) as Promise<bigint>;
  }

  async executeRequestSlashing({
    payoutAmount,
    capacityFactor,
    reason,
  }: {
    payoutAmount: number;
    capacityFactor: number;
    reason: string;
  }) {
    // get total staked
    let totalVaultBalance = await this.queryTotalVaultStakedAmount();

    if (payoutAmount > totalVaultBalance) {
      throw new Error("Payout amount exceeds total vault balance");
    }

    let mbips = (BigInt(payoutAmount) * 10_000_000n) / totalVaultBalance;

    if (mbips > capacityFactor * 10_000_000) {
      throw new Error("Bips to slash exceeds max bips");
    }
    if (mbips < 1) {
      throw new Error("Bips to slash must be at least 1");
    }

    let currentBlock = await this.client.getBlock();
    // set offending timestamp to 5 seconds before current block timestamp
    let timestamp = currentBlock.timestamp - 5n;

    // request slashing uses block.timestamp, so if next block is mined with a different timestamp, the request id will differ
    let requestSlashingRes = await this.router.simulate.requestSlashing(
      [
        {
          operator: this.operator,
          mbips,
          timestamp,
          reason,
        },
      ],
      { account: this.service },
    );

    await this.router.write.requestSlashing(requestSlashingRes.request);
    return requestSlashingRes.result as `0x${string}`;
  }

  async executeLockSlashing(slashingRequestId: string): Promise<any> {
    return this.router.write.lockSlashing([slashingRequestId], { account: this.service });
  }

  async executeFinalizeSlashing(slashingRequestId: string): Promise<any> {
    return this.router.write.finalizeSlashing([slashingRequestId], { account: this.service });
  }

  async executeDistributeRewards({
    token,
    amount,
    merkleRoot,
  }: {
    token: string;
    amount: number;
    merkleRoot: `0x${string}`;
  }): Promise<any> {
    return this.rewards.write.distributeRewards([token, amount, merkleRoot], { account: this.service });
  }
}
