import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice, IndexedTx, StargateClient } from "@cosmjs/stargate";
import { AbstractStartedContainer, GenericContainer, StartedTestContainer, Wait } from "testcontainers";

import { deploy, deployCw20 } from "./deployer";

export class SatLayerContainer extends GenericContainer {
  constructor(image = "cosmwasm/wasmd:v0.55.0") {
    super(image);
    this.withEnvironment({ CHAIN_ID: "wasm-1337" });
    this.withCommand(["/opt/setup_and_run.sh"]);
    this.withExposedPorts(26657);
    this.withWaitStrategy(Wait.forLogMessage("indexed block events"));
  }

  public override async start(): Promise<StartedSatLayerContainer> {
    return new StartedSatLayerContainer(await super.start());
  }
}

export interface Contracts {
  pauser: string;
  registry: string;
  router: string;
  cw20: string;
  vaults: {
    cw20: string;
    bank: string;
  };
}

export class StartedSatLayerContainer extends AbstractStartedContainer {
  constructor(startedTestContainer: StartedTestContainer) {
    super(startedTestContainer);
  }

  getHostRpcUrl(): string {
    const host = this.startedTestContainer.getHost();
    const port = this.startedTestContainer.getMappedPort(26657);
    return `tcp://${host}:${port}/`;
  }

  async fund(address: string, coin: string): Promise<IndexedTx> {
    const result = await this.exec([
      "/bin/sh",
      "-c",
      [
        "echo",
        "1234567890",
        "|",
        "wasmd",
        "tx",
        "bank",
        "send",
        "validator",
        address,
        coin,
        "--chain-id",
        "wasm-1337",
        "-y",
        "-o",
        "json",
      ].join(" "),
    ]);

    const txId = JSON.parse(result.output).txhash;
    return this.waitForTx(txId);
  }

  async waitForTx(txId: string, timeout: number = 5000): Promise<IndexedTx> {
    const client = await StargateClient.connect(this.getHostRpcUrl());
    const startTime = Date.now();

    while (Date.now() - startTime < timeout) {
      const tx = await client.getTx(txId);
      if (tx !== null) {
        return tx;
      }

      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    throw new Error(`Transaction ${txId} not found within timeout of ${timeout}ms`);
  }

  async bootstrap(): Promise<Contracts> {
    const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon cactus";
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "wasm",
    });

    const endpoint = this.getHostRpcUrl();
    const client = await SigningCosmWasmClient.connectWithSigner(endpoint, wallet, {
      gasPrice: GasPrice.fromString("0.002000ustake"),
    });

    const admin = (await wallet.getAccounts())[0].address;
    await this.fund(admin, "100000000ustake");
    return this.deployAll(admin, client);
  }

  private async deployAll(from: string, client: SigningCosmWasmClient): Promise<Contracts> {
    const pauser = await deploy(client, from, "@satlayer/bvs-pauser", {
      initial_paused: false,
      owner: from,
    });

    const registry = await deploy(client, from, "@satlayer/bvs-registry", {
      owner: from,
      pauser: pauser.address,
    });

    const router = await deploy(client, from, "@satlayer/bvs-vault-router", {
      owner: from,
      pauser: pauser.address,
      registry: registry.address,
    });

    const cw20 = await deployCw20(client, from, {
      decimals: 8,
      name: "Test Bitcoin",
      symbol: "tBTC",
      initial_balances: [
        {
          address: from,
          amount: "100000000000",
        },
      ],
    });

    const vaultCw20 = await deploy(client, from, "@satlayer/bvs-vault-cw20", {
      pauser: pauser.address,
      router: router.address,
      operator: from,
      cw20_contract: cw20.address,
    });

    const vaultBank = await deploy(client, from, "@satlayer/bvs-vault-bank", {
      denom: "ustake",
      pauser: pauser.address,
      router: router.address,
      operator: from,
    });

    await client.executeMultiple(
      from,
      [
        {
          contractAddress: router.address,
          msg: {
            set_vault: {
              vault: vaultCw20.address,
              whitelisted: true,
            },
          },
        },
        {
          contractAddress: router.address,
          msg: {
            set_vault: {
              vault: vaultBank.address,
              whitelisted: true,
            },
          },
        },
      ],
      "auto",
      undefined,
    );

    return {
      pauser: pauser.address,
      registry: registry.address,
      router: router.address,
      cw20: cw20.address,
      vaults: {
        cw20: vaultCw20.address,
        bank: vaultBank.address,
      },
    };
  }
}
