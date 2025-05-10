import { IndexedTx, StargateClient } from "@cosmjs/stargate";
import { AbstractStartedContainer, GenericContainer, StartedTestContainer, Wait } from "testcontainers";

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
}

// TODO(fuxingloh): deploy bvs contracts
