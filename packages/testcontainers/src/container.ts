import { join } from "node:path";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Slip10RawIndex } from "@cosmjs/crypto";
import { DirectSecp256k1HdWallet, parseCoins } from "@cosmjs/proto-signing";
import { DeliverTxResponse, GasPrice } from "@cosmjs/stargate";
import { AbstractStartedContainer, GenericContainer, StartedTestContainer, Wait } from "testcontainers";

export class CosmWasmContainer extends GenericContainer {
  constructor(image = "cosmwasm/wasmd:v0.55.0") {
    super(image);
    this.withCopyDirectoriesToContainer([
      {
        source: join(__dirname, "../wasmd"),
        target: "/root/.wasmd",
      },
    ]);
    this.withCommand(["wasmd", "start", "--rpc.laddr", "tcp://0.0.0.0:26657", "--trace"]);
    this.withExposedPorts(26657);
    this.withWaitStrategy(Wait.forLogMessage("indexed block events"));
  }

  public override async start(): Promise<StartedCosmWasmContainer> {
    const started = await super.start();
    const rpcEndpoint = `tcp://${started.getHost()}:${started.getMappedPort(26657)}/`;

    // HD Paths m/0' to m/9' are funded for this mnemonic.
    const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon cactus";
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "wasm",
      hdPaths: Array.from({ length: 10 }, (_, i) => [Slip10RawIndex.hardened(i)]),
    });

    const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, {
      gasPrice: GasPrice.fromString("0.002000ustake"),
      broadcastPollIntervalMs: 200,
    });

    return new StartedCosmWasmContainer(started, wallet, client);
  }
}

export class StartedCosmWasmContainer extends AbstractStartedContainer {
  constructor(
    startedTestContainer: StartedTestContainer,
    public readonly wallet: DirectSecp256k1HdWallet,
    public readonly client: SigningCosmWasmClient,
  ) {
    super(startedTestContainer);
  }

  getRpcEndpoint(): string {
    const host = this.startedTestContainer.getHost();
    const port = this.startedTestContainer.getMappedPort(26657);
    return `tcp://${host}:${port}/`;
  }

  async fund(address: string, coins: string): Promise<DeliverTxResponse> {
    const [account] = await this.wallet.getAccounts();
    return this.client.sendTokens(account.address, address, parseCoins(coins), "auto");
  }
}
