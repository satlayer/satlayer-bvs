import { readFile, writeFile } from "node:fs/promises";
import { setTimeout } from "node:timers/promises";
import { StartedAnvilContainer, SuperTestClient } from "@satlayer/testcontainers";
import { Account, decodeEventLog, getContract, GetContractReturnType, Log } from "viem";
import { abi } from "../src/contracts/out/BVS.sol/BVS.json";

/**
 * This function is an example of an expensive computation.
 * You want to perform this off-chain to reduce on-chain costs.
 */
function compute(input: number): number {
  return input * input;
}

/**
 * A long-running process that listens for Requests and Responds to them.
 * It is an artificial node supposed to be run by an operator.
 */
export class SquaringNode {
  private running: boolean = true;

  constructor(
    private readonly client: SuperTestClient,
    private readonly anvil: StartedAnvilContainer,
    private readonly operator: Account,
    private readonly bvsContract: GetContractReturnType<typeof abi, SuperTestClient, `0x${string}`>,
  ) {}

  public async start(startFrom: bigint) {
    let progress = { height: startFrom };

    const unwatch = this.client.watchContractEvent({
      address: this.bvsContract.address,
      abi: abi,
      eventName: "Requested",
      onLogs: async (logs) => {
        logs.forEach(async (log) => {
          console.log("New Request:", log);
          await this.respondToRequest(Number(log.args.input));
          await this.anvil.mineBlock(1);
        });
      },
    });

    while (this.running) {
      const currentHeight = await this.client.getBlockNumber({ cacheTime: 0 });

      if (progress.height <= currentHeight) {
        continue;
      } else {
        // No new blocks, sleep for 1 second
        await setTimeout(1000);
      }
      progress.height = currentHeight;
    }
    unwatch();
  }

  private async respondToRequest(input: number): Promise<void> {
    const response = compute(input);

    // Respond to the request
    await this.bvsContract.write.respond([input, response], {
      account: this.operator.address,
    });
  }

  public async stop() {
    this.running = false;
  }
}

/**
 * This class is used to interact with the BVS contract.
 * An artificial (supposed) service node that can be used to request computations.
 * Run by service
 */
export class ServiceNode {
  constructor(private readonly bvsContract: GetContractReturnType<typeof abi, any, `0x${string}`>) {}

  /**
   * Request a computation to be performed by the service node.
   */
  public async request(sender: string, input: number): Promise<string> {
    return this.bvsContract.write.request([input], { account: sender });
  }

  /**
   * Get the response for a given input uploaded by an operator.
   */
  public async getResponse(input: number, operator: string): Promise<bigint> {
    return this.bvsContract.read.getResponse([input, operator]);
  }
}
