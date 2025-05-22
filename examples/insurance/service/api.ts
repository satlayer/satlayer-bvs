import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { ExecuteMsg as RouterExecuteMsg, QueryMsg as RouterQueryMsg } from "@satlayer/cosmwasm-schema/vault-router";
import { QueryMsg as VaultBankQueryMsg } from "@satlayer/cosmwasm-schema/vault-bank";

interface ApiServiceOptions {
  client: SigningCosmWasmClient;
  vault: string;
  registry: string;
  router: string;
  operator: string;
  service: string;
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

  constructor({ client, vault, registry, router, operator, service }: ApiServiceOptions) {
    this.client = client;
    this.vault = vault;
    this.registry = registry;
    this.router = router;
    this.operator = operator;
    this.service = service;
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

  async getStakedAmount(): Promise<number> {
    let vaultBalanceMsg: VaultBankQueryMsg = {
      total_assets: {},
    };
    return await this.client.queryContractSmart(this.vault, vaultBalanceMsg);
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
    let totalVaultBalance = await this.getStakedAmount();

    if (payoutAmount > totalVaultBalance) {
      throw new Error("Payout amount exceeds total vault balance");
    }

    let bips = (payoutAmount / totalVaultBalance) * 10000;

    if (bips > capacityFactor * 10000) {
      throw new Error("Bips to slash exceeds max bips");
    }

    let timestamp = Math.floor(Date.now() - 1000) * 1_000_000;
    console.log("slashing timestamp: ", timestamp);

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

  async getSlashingRequestId({ service, operator }: { service: string; operator: string }): Promise<string> {
    let slashingRequestIdMsg: RouterQueryMsg = {
      slashing_request_id: {
        service,
        operator,
      },
    };
    return await this.client.queryContractSmart(this.router, slashingRequestIdMsg);
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
}
