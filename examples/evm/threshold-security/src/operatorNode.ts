import { StartedAnvilContainer, SuperTestClient } from "@satlayer/testcontainers";
import { Account, getContract, GetContractReturnType, WatchContractEventReturnType } from "viem";
import { abi } from "./out/BVS.sol/BVS.json";

export class OperatorNode {
  private unwatch: WatchContractEventReturnType | undefined;

  private bvsContract: GetContractReturnType<typeof abi, SuperTestClient, `0x${string}`>;

  constructor(
    private readonly label: string,
    private readonly client: SuperTestClient,
    private readonly anvil: StartedAnvilContainer,
    private readonly operator: Account,
    private readonly bvsAddress: `0x${string}`,
  ) {
    this.bvsContract = getContract({
      address: bvsAddress,
      abi: abi,
      client,
    });
  }

  /**
   * Operator listen for request event and respond.
   */
  public start() {
    console.log(`[${this.label}] Start listening for Requested event...`);
    this.unwatch = this.client.watchContractEvent({
      address: this.bvsAddress,
      abi: abi,
      eventName: "Requested",
      onLogs: async (logs) => {
        await Promise.all(logs.map((log) => this.respondToRequest(log.args.requestId, log.args.input)));
      },
    });
  }

  private verify(input: bigint): bigint {
    return input * input;
  }

  private async respondToRequest(requestId: bigint, input: bigint): Promise<void> {
    console.log(`[${this.label}] Verifying requestId: ${requestId} input: ${input}...`);
    const response = this.verify(input);

    console.log(`[${this.label}] Verified requestId: ${requestId} input: ${input} response: ${response}`);
    // Respond to the request
    await this.bvsContract.write.respond([requestId, response], {
      account: this.operator.address,
    });
  }

  public stop() {
    this.unwatch && this.unwatch();
  }
}
