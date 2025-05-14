import { Event, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { ExecuteMsg, GetResponseResponse, QueryMsg } from "./contract";
import { readFile, writeFile } from "node:fs/promises";
import { setTimeout } from "node:timers/promises";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate/build/signingcosmwasmclient";

/**
 * This function is an example of an expensive computation.
 * You want to perform this off-chain to reduce on-chain costs.
 */
function compute(input: number): number {
  return input * input;
}

/**
 * A long-running process that listens for Requests and Responds to them.
 */
export class SquaringNode {
  private running: boolean = true;

  constructor(
    private readonly client: SigningCosmWasmClient,
    private readonly contract: string,
    private readonly operator: string,
  ) {}

  public async start(startFrom: number) {
    let progress = { height: startFrom };

    while (this.running) {
      const currentHeight = await this.client.getHeight();

      if (progress.height < currentHeight) {
        // Process all blocks from height to currentHeight
        for (let h = progress.height; h < currentHeight && this.running; h++) {
          const events = await this.getRequests(this.contract, h);

          for (const event of events) {
            // Find the input attribute
            const inputAttr = event.attributes.find((attr) => attr.key === "input");
            if (inputAttr) {
              const input = parseInt(inputAttr.value);
              await this.respond(input);
            }
          }

          console.log(`Processed block ${h}, found ${events.length} events for contract ${this.contract}`);
          // Update progress after processing each height
          progress.height = h + 1;
        }
      } else {
        // No new blocks, sleep for 1 second
        await setTimeout(1000);
      }
    }
  }

  /**
   * Find all Request events for a given contract at a given block height.
   */
  private async getRequests(contract: string, height: number): Promise<Event[]> {
    const events: Event[] = [];
    for (const tx of await this.client.searchTx(`tx.height=${height}`)) {
      for (const event of tx.events) {
        if (
          event.type === "wasm" &&
          event.attributes.some((attr) => attr.key === "_contract_address" && attr.value === contract) &&
          event.attributes.some((attr) => attr.key === "method" && attr.value === "Request")
        ) {
          events.push(event);
        }
      }
    }
    return events;
  }

  /**
   * Respond to a request with the result of the computation.
   */
  private async respond(input: number): Promise<void> {
    const output = compute(input);

    const msg: ExecuteMsg = {
      respond: {
        input,
        output,
      },
    };

    await this.client.execute(this.operator, this.contract, msg, "auto");
  }

  public async stop() {
    this.running = false;
  }
}

export class ServiceNode {
  constructor(
    private readonly client: SigningCosmWasmClient,
    private readonly contract: string,
  ) {}

  /**
   * Request a computation to be performed by the service node.
   */
  public async request(sender: string, input: number): Promise<ExecuteResult> {
    const msg: ExecuteMsg = {
      request: {
        input,
      },
    };

    return this.client.execute(sender, this.contract, msg, "auto");
  }

  /**
   * Get the response for a given input uploaded by an operator.
   */
  public async getResponse(input: number, operator: string): Promise<GetResponseResponse> {
    const msg: QueryMsg = {
      get_response: {
        input,
        operator,
      },
    };
    return await this.client.queryContractSmart(this.contract, msg);
  }
}
