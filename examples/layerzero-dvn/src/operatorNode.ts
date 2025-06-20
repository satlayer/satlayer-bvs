import { Event, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { setTimeout } from "node:timers/promises";
import { ExecuteMsg, GetPacketResponse, PacketClass } from "./bvs-dvn-contract";
import { encodePacked, keccak256 } from "viem";
import { trim0x } from "@layerzerolabs/lz-v2-utilities";

export class OperatorNode {
  private running: boolean = true;

  private readonly label: string = "OperatorNode";

  private readonly client: SigningCosmWasmClient;
  private readonly bvsAddress: string;
  private readonly operatorAddress: string;

  constructor(config: { client: SigningCosmWasmClient; bvsAddress: string; operatorAddress: string; label?: string }) {
    this.client = config.client;
    this.bvsAddress = config.bvsAddress;
    this.operatorAddress = config.operatorAddress;
    this.label = config.label || this.label;
  }

  /**
   * Start the Operator node that watches the BVS contract for packets broadcasted.
   * When a packet is broadcasted,
   * it will verify the packet and submit it to the BVS contract along with the payload hash.
   * @param startFrom
   */
  public async start(startFrom: number) {
    let progress = { height: startFrom };

    while (this.running) {
      const currentHeight = await this.client.getHeight();

      if (progress.height < currentHeight) {
        // Process all blocks from height to currentHeight
        for (let h = progress.height; h < currentHeight && this.running; h++) {
          const events = await this.getPacketBroadcastEvent(this.bvsAddress, h);

          for (const event of events) {
            // Find the input attribute
            const packet_guid = event.attributes.find((attr) => attr.key === "packet_guid");
            if (packet_guid) {
              const packet = await this.getPacket(packet_guid.value);
              if (packet !== null) {
                let payload_hash = await this.verifyPacket(packet.packet);
                await this.submitPacket(packet.packet.guid, payload_hash);
                console.log(
                  `[${this.label}] Submitted packet with guid ${packet.packet.guid} and payload hash ${payload_hash}`,
                );
              }
            }
          }
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
   * Get all PacketBroadcast events in BVS contract at a given block height.
   * @param contract
   * @param height
   * @private
   */
  private async getPacketBroadcastEvent(contract: string, height: number): Promise<Event[]> {
    const events: Event[] = [];
    for (const tx of await this.client.searchTx(`tx.height=${height}`)) {
      for (const event of tx.events) {
        if (
          event.type === "wasm" &&
          event.attributes.some((attr) => attr.key === "_contract_address" && attr.value === contract) &&
          event.attributes.some((attr) => attr.key === "method" && attr.value === "BroadcastPacket")
        ) {
          events.push(event);
        }
      }
    }
    return events;
  }

  /**
   * Get the packet details from the BVS contract using the packet guid.
   * @param guid
   * @private
   */
  private async getPacket(guid: string): Promise<GetPacketResponse | null> {
    let res = await this.client.queryContractSmart(this.bvsAddress, {
      get_packet: guid,
    });

    return res as GetPacketResponse | null;
  }

  /**
   * Verify the packet by calculating the payload hash.
   * @param packet
   * @private
   */
  private async verifyPacket(packet: PacketClass): Promise<string> {
    const { guid, message } = packet;

    // ** Implement your packet verification logic here **

    // For this case, we will do a keccak256 hash (guid, message)
    let payload = keccak256(
      encodePacked(["bytes32", "bytes"], [`0x${guid}` as `0x${string}`, `0x${message}` as `0x${string}`]),
    );
    return trim0x(payload);
  }

  private async submitPacket(guid: string, payload_hash: string) {
    let submitPacketMsg: ExecuteMsg = {
      submit_packet: {
        packet_guid: guid,
        payload_hash: payload_hash,
      },
    };
    return this.client.execute(this.operatorAddress, this.bvsAddress, submitPacketMsg, "auto");
  }
}
