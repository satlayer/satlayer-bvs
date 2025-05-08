import { AbstractStartedContainer, GenericContainer, StartedTestContainer, Wait } from "testcontainers";

const CHAIN_ID = "sats-1337";

export class SatLayerContainer extends GenericContainer {
  constructor(image = "cosmwasm/wasmd:v0.55.0") {
    super(image);
    this.withCommand(["wasmd", "testnet", "start", "--chain-id", CHAIN_ID, "--rpc.address", "tcp://0.0.0.0:26657"]);
    this.withExposedPorts(26657);
    this.withWaitStrategy(Wait.forLogMessage("press the Enter Key to terminate"));
    this.createOpts = {
      ...this.createOpts,
      OpenStdin: true,
    };
  }

  public override async start(): Promise<StartedSatLayerContainer> {
    return new StartedSatLayerContainer(await super.start());
  }
}

export class StartedSatLayerContainer extends AbstractStartedContainer {
  constructor(startedTestContainer: StartedTestContainer) {
    super(startedTestContainer);
  }

  public getChainId(): string {
    return CHAIN_ID;
  }

  public getHostRpcUrl(): string {
    const host = this.startedTestContainer.getHost();
    const port = this.startedTestContainer.getMappedPort(26657);
    return `tcp://${host}:${port}/`;
  }
}
