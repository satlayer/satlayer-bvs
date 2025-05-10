import { Event, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { ExecuteMsg } from "./contract";
import { readFile, writeFile } from "node:fs/promises";
import { setTimeout } from "node:timers/promises";

function compute(input: number): number {
  return input * input;
}

export class Node {
  private running: boolean = true;

  constructor(
    private readonly contract: string,
    private readonly client: SigningCosmWasmClient,
    private readonly operator: string,
  ) {}

  public async start() {
    let progress: { height: number };

    try {
      progress = JSON.parse(await readFile("progress.json", "utf-8"));
    } catch (error) {
      // File doesn't exist or can't be read, start from scratch
      console.log("No progress.json found, starting from scratch");
      progress = { height: 0 };
    }

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

          // Update progress after processing each height
          progress.height = h + 1;
          await writeFile("progress.json", JSON.stringify(progress), "utf-8");
        }
      } else {
        // No new blocks, sleep for 1 second
        await setTimeout(1000);
      }
    }
  }

  private async getRequests(contract: string, height: number): Promise<Event[]> {
    const events: Event[] = [];
    for (const tx of await this.client.searchTx(`tx.height=${height}`)) {
      for (const event of tx.events) {
        if (
          event.attributes.some((attr) => attr.key === "_contract_address" && attr.value === contract) &&
          event.attributes.some((attr) => attr.key === "method" && attr.value === "request")
        ) {
          events.push(event);
        }
      }
    }
    return events;
  }

  public async respond(input: number): Promise<void> {
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
