import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { instantiateBvs, uploadBvs } from "./bvs";
import { StartedCosmWasmContainer } from "./container";
import { Cw20InitMsg, instantiateCw20, uploadCw20 } from "./cw20";

type State = {
  cw20: { codeId: number };
  pauser: { codeId: number; address: string };
  registry: { codeId: number; address: string };
  router: { codeId: number; address: string };
  vaultCw20: { codeId: number };
  vaultBank: { codeId: number };
};

export class SatLayerContracts {
  private constructor(
    public readonly started: StartedCosmWasmContainer,
    public readonly state: State,
  ) {}

  get client(): SigningCosmWasmClient {
    return this.started.client;
  }

  get wallet(): DirectSecp256k1HdWallet {
    return this.started.wallet;
  }

  static async bootstrap(started: StartedCosmWasmContainer): Promise<SatLayerContracts> {
    const accounts = await started.wallet.getAccounts();
    const [cw20Upload, pauserUpload, registryUpload, routerUpload, vaultCw20Upload, vaultBankUpload] =
      await Promise.all([
        uploadCw20(started.client, accounts[0].address),
        uploadBvs(started.client, accounts[1].address, "@satlayer/bvs-pauser"),
        uploadBvs(started.client, accounts[2].address, "@satlayer/bvs-registry"),
        uploadBvs(started.client, accounts[3].address, "@satlayer/bvs-vault-router"),
        uploadBvs(started.client, accounts[4].address, "@satlayer/bvs-vault-cw20"),
        uploadBvs(started.client, accounts[5].address, "@satlayer/bvs-vault-bank"),
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

    const routerResult = await instantiateBvs(
      started.client,
      accounts[0].address,
      "@satlayer/bvs-vault-router",
      routerUpload.codeId,
      {
        owner: accounts[0].address,
        pauser: pauserResult.contractAddress,
        registry: registryResult.contractAddress,
      },
    );

    return new SatLayerContracts(started, {
      cw20: { codeId: cw20Upload.codeId },
      pauser: { address: pauserResult.contractAddress, codeId: pauserUpload.codeId },
      registry: { address: registryResult.contractAddress, codeId: registryUpload.codeId },
      router: { address: routerResult.contractAddress, codeId: routerUpload.codeId },
      vaultBank: { codeId: vaultBankUpload.codeId },
      vaultCw20: { codeId: vaultCw20Upload.codeId },
    });
  }

  async initCw20(initMsg: Cw20InitMsg): Promise<string> {
    const sender = (await this.wallet.getAccounts())[0].address;
    const result = await instantiateCw20(this.client, sender, this.state.cw20.codeId, initMsg);
    return result.contractAddress;
  }

  async initVaultCw20(operator: string, cw20_contract: string): Promise<string> {
    const sender = (await this.wallet.getAccounts())[0].address;
    const codeId = this.state.vaultCw20.codeId;
    const vaultCw20 = await instantiateBvs(this.client, sender, "@satlayer/bvs-vault-cw20", codeId, {
      operator: operator,
      cw20_contract: cw20_contract,
      pauser: this.state.pauser.address,
      router: this.state.router.address,
    });

    await this.client.execute(
      sender,
      this.state.router.address,
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
    const codeId = this.state.vaultBank.codeId;
    const vaultBank = await instantiateBvs(this.client, sender, "@satlayer/bvs-vault-bank", codeId, {
      operator: operator,
      denom: denom,
      pauser: this.state.pauser.address,
      router: this.state.router.address,
    });

    await this.client.execute(
      sender,
      this.state.router.address,
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
