import { CosmWasmClient, Event, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { ExecuteMsg } from "./contract";
import { readFile } from "node:fs/promises";

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
    const status: {} = JSON.parse(await readFile("progress.json", "utf-8"));

    let height = await this.client.getHeight();

    while (this.running) {}
  }

  private async getRequestEvents(contract: string, height: number): Promise<Event[]> {
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
