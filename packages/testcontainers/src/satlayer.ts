import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { ExecuteMsg as RegistryExecuteMsg, QueryMsg as RegistryQueryMsg } from "@satlayer/cosmwasm-schema/registry";
import { ExecuteMsg as RouterExecuteMsg, QueryMsg as RouterQueryMsg } from "@satlayer/cosmwasm-schema/vault-router";

import { instantiateBvs, uploadBvs } from "./bvs";
import { StartedCosmWasmContainer } from "./container";
import { Cw20InitMsg, instantiateCw20, uploadCw20 } from "./cw20";

type Data = {
  cw20: { codeId: number };
  pauser: { codeId: number; address: string };
  registry: { codeId: number; address: string };
  guardrail: { codeId: number; address: string };
  router: { codeId: number; address: string };
  vaultCw20: { codeId: number };
  vaultCw20Tokenized: { codeId: number };
  vaultBank: { codeId: number };
  vaultBankTokenized: { codeId: number };
};

export class SatLayerContracts {
  private constructor(
    public readonly started: StartedCosmWasmContainer,
    public readonly data: Data,
    public readonly pauser = new Pauser(started, data.pauser.address),
    public readonly registry = new Registry(started, data.registry.address),
    public readonly guardrail = new Router(started, data.guardrail.address),
    public readonly router = new Router(started, data.router.address),
  ) {}

  get client(): SigningCosmWasmClient {
    return this.started.client;
  }

  get wallet(): DirectSecp256k1HdWallet {
    return this.started.wallet;
  }

  static async bootstrap(started: StartedCosmWasmContainer): Promise<SatLayerContracts> {
    const accounts = await started.wallet.getAccounts();
    const [
      cw20Upload,
      pauserUpload,
      registryUpload,
      routerUpload,
      vaultCw20Upload,
      vaultBankUpload,
      guardrailUpload,
      vaultCw20TokenizedUpload,
      vaultBankTokenizedUpload,
    ] = await Promise.all([
      uploadCw20(started.client, accounts[0].address),
      uploadBvs(started.client, accounts[1].address, "@satlayer/bvs-pauser"),
      uploadBvs(started.client, accounts[2].address, "@satlayer/bvs-registry"),
      uploadBvs(started.client, accounts[3].address, "@satlayer/bvs-vault-router"),
      uploadBvs(started.client, accounts[4].address, "@satlayer/bvs-vault-cw20"),
      uploadBvs(started.client, accounts[5].address, "@satlayer/bvs-vault-bank"),
      uploadBvs(started.client, accounts[6].address, "@satlayer/bvs-guardrail"),
      uploadBvs(started.client, accounts[7].address, "@satlayer/bvs-vault-cw20-tokenized"),
      uploadBvs(started.client, accounts[8].address, "@satlayer/bvs-vault-bank-tokenized"),
    ]);

    const pauserResult = await instantiateBvs(
      started.client,
      accounts[0].address,
      "@satlayer/bvs-pauser",
      pauserUpload.codeId,
      {
        initial_paused: false,
        owner: accounts[0].address,
      },
    );

    const registryResult = await instantiateBvs(
      started.client,
      accounts[0].address,
      "@satlayer/bvs-registry",
      registryUpload.codeId,
      {
        owner: accounts[0].address,
        pauser: pauserResult.contractAddress,
      },
    );

    const guardrailResult = await instantiateBvs(
      started.client,
      accounts[0].address,
      "@satlayer/bvs-guardrail",
      guardrailUpload.codeId,
      {
        owner: accounts[0].address,
        threshold: { absolute_percentage: { percentage: "0" } },
        default_expiration: 1000,
        members: [{ addr: accounts[0].address, weight: 1 }],
      },
    );

    const routerResult = await instantiateBvs(
      started.client,
      accounts[0].address,
      "@satlayer/bvs-vault-router",
      routerUpload.codeId,
      {
        owner: accounts[0].address,
        pauser: pauserResult.contractAddress,
        registry: registryResult.contractAddress,
        guardrail: guardrailResult.contractAddress,
      },
    );

    return new SatLayerContracts(started, {
      cw20: { codeId: cw20Upload.codeId },
      pauser: { address: pauserResult.contractAddress, codeId: pauserUpload.codeId },
      registry: { address: registryResult.contractAddress, codeId: registryUpload.codeId },
      guardrail: { address: guardrailResult.contractAddress, codeId: guardrailUpload.codeId },
      router: { address: routerResult.contractAddress, codeId: routerUpload.codeId },
      vaultBank: { codeId: vaultBankUpload.codeId },
      vaultBankTokenized: { codeId: vaultBankTokenizedUpload.codeId },
      vaultCw20: { codeId: vaultCw20Upload.codeId },
      vaultCw20Tokenized: { codeId: vaultCw20TokenizedUpload.codeId },
    });
  }

  async initCw20(initMsg: Cw20InitMsg): Promise<string> {
    const sender = (await this.wallet.getAccounts())[0].address;
    const result = await instantiateCw20(this.client, sender, this.data.cw20.codeId, initMsg);
    return result.contractAddress;
  }

  async initVaultCw20(operator: string, cw20_contract: string): Promise<string> {
    const sender = (await this.wallet.getAccounts())[0].address;
    const codeId = this.data.vaultCw20.codeId;
    const vaultCw20 = await instantiateBvs(this.client, sender, "@satlayer/bvs-vault-cw20", codeId, {
      operator: operator,
      cw20_contract: cw20_contract,
      pauser: this.data.pauser.address,
      router: this.data.router.address,
    });

    await this.client.execute(
      sender,
      this.data.router.address,
      {
        set_vault: {
          vault: vaultCw20.contractAddress,
          whitelisted: true,
        },
      },
      "auto",
    );

    return vaultCw20.contractAddress;
  }

  async initVaultCw20Tokenized(operator: string, cw20_contract: string, symbol = "satCWT"): Promise<string> {
    const sender = (await this.wallet.getAccounts())[0].address;
    const codeId = this.data.vaultCw20Tokenized.codeId;
    const vaultCw20 = await instantiateBvs(this.client, sender, "@satlayer/bvs-vault-cw20-tokenized", codeId, {
      operator: operator,
      cw20_contract: cw20_contract,
      pauser: this.data.pauser.address,
      router: this.data.router.address,
      name: symbol,
      symbol: symbol,
    });

    await this.client.execute(
      sender,
      this.data.router.address,
      {
        set_vault: {
          vault: vaultCw20.contractAddress,
          whitelisted: true,
        },
      },
      "auto",
    );

    return vaultCw20.contractAddress;
  }

  async initVaultBank(operator: string, denom: string): Promise<string> {
    const sender = (await this.wallet.getAccounts())[0].address;
    const codeId = this.data.vaultBank.codeId;
    const vaultBank = await instantiateBvs(this.client, sender, "@satlayer/bvs-vault-bank", codeId, {
      operator: operator,
      denom: denom,
      pauser: this.data.pauser.address,
      router: this.data.router.address,
    });

    await this.client.execute(
      sender,
      this.data.router.address,
      {
        set_vault: {
          vault: vaultBank.contractAddress,
          whitelisted: true,
        },
      },
      "auto",
    );

    return vaultBank.contractAddress;
  }

  async initVaultBankTokenized(operator: string, denom: string, symbol = "satBANKT"): Promise<string> {
    const sender = (await this.wallet.getAccounts())[0].address;
    const codeId = this.data.vaultBankTokenized.codeId;
    const vaultBank = await instantiateBvs(this.client, sender, "@satlayer/bvs-vault-bank-tokenized", codeId, {
      operator: operator,
      denom: denom,
      pauser: this.data.pauser.address,
      router: this.data.router.address,
      decimals: 6,
      name: symbol,
      symbol: symbol,
    });

    await this.client.execute(
      sender,
      this.data.router.address,
      {
        set_vault: {
          vault: vaultBank.contractAddress,
          whitelisted: true,
        },
      },
      "auto",
    );

    return vaultBank.contractAddress;
  }
}

export class Pauser {
  constructor(
    public readonly started: StartedCosmWasmContainer,
    public readonly address: string,
  ) {}
}

export class Registry {
  constructor(
    public readonly started: StartedCosmWasmContainer,
    public readonly address: string,
  ) {}

  async execute(client: SigningCosmWasmClient, sender: string, executeMsg: RegistryExecuteMsg) {
    return client.execute(sender, this.address, executeMsg, "auto");
  }

  async query(queryMsg: RegistryQueryMsg): Promise<any> {
    return await this.started.client.queryContractSmart(this.address, queryMsg);
  }
}

export class Router {
  constructor(
    public readonly started: StartedCosmWasmContainer,
    public readonly address: string,
  ) {}

  async execute(client: SigningCosmWasmClient, sender: string, executeMsg: RouterExecuteMsg) {
    return client.execute(sender, this.address, executeMsg, "auto");
  }

  async query(queryMsg: RouterQueryMsg): Promise<any> {
    return await this.started.client.queryContractSmart(this.address, queryMsg);
  }
}
