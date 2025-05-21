import { join } from "node:path";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Slip10RawIndex } from "@cosmjs/crypto";
import { DirectSecp256k1HdWallet, OfflineSigner, parseCoins } from "@cosmjs/proto-signing";
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
    const endpoint = `tcp://${started.getHost()}:${started.getMappedPort(26657)}/`;

    const wallet = await CosmWasmContainer.getDefaultWallet();
    const client = await CosmWasmContainer.getDefaultClient(endpoint, wallet);
    return new StartedCosmWasmContainer(started, wallet, client);
  }

  /**
   * HD Paths m/0' to m/9' are funded for this mnemonic.
   */
  static async getDefaultWallet(): Promise<DirectSecp256k1HdWallet> {
    const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon cactus";
    return await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "wasm",
      hdPaths: Array.from({ length: 10 }, (_, i) => [Slip10RawIndex.hardened(i)]),
    });
  }

  static async getDefaultClient(endpoint: string, wallet: DirectSecp256k1HdWallet): Promise<SigningCosmWasmClient> {
    return await SigningCosmWasmClient.connectWithSigner(endpoint, wallet, {
      gasPrice: GasPrice.fromString("0.002000ustake"),
      broadcastPollIntervalMs: 200,
    });
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
    const host = this.getHost();
    const port = this.getMappedPort(26657);
    return `http://${host}:${port}`;
  }

  getRpcPort(): number {
    return 26657;
  }

  getChainId() {
    return "wasm-1337";
  }

  async fund(coins: string, ...addresses: string[]): Promise<DeliverTxResponse> {
    const [from] = await this.wallet.getAccounts();
    return this.client.signAndBroadcast(
      from.address,
      addresses.map((address: string) => {
        return {
          typeUrl: "/cosmos.bank.v1beta1.MsgSend",
          value: {
            fromAddress: from.address,
            toAddress: address,
            amount: parseCoins(coins),
          },
        };
      }),
      "auto",
    );
  }

  /**
   * Creates a new signer configured to use this container as the RPC endpoint.
   * @param wallet
   */
  async newSigner(wallet: OfflineSigner) {
    return await SigningCosmWasmClient.connectWithSigner(this.getRpcEndpoint(), wallet, {
      gasPrice: GasPrice.fromString("0.002000ustake"),
      broadcastPollIntervalMs: 200,
    });
  }
}
