import { setTimeout } from "node:timers/promises";
import { encodePacked, getContract, GetContractReturnType, keccak256 } from "viem";
import { SuperTestClient } from "@satlayer/testcontainers";
import bvs from "./contracts/out/BVS.sol/BVS.json";

export class OperatorNode {
  private running: boolean = true;

  private readonly label: string = "OperatorNode";

  private readonly client: SuperTestClient;
  private readonly bvsAddress: `0x${string}`;
  private readonly bvsContract: GetContractReturnType<typeof bvs.abi, SuperTestClient, `0x${string}`>;
  private readonly operatorAddress: string;

  constructor(config: { client: SuperTestClient; bvsAddress: `0x${string}`; operatorAddress: string; label?: string }) {
    this.client = config.client;
    this.bvsAddress = config.bvsAddress;
    this.bvsContract = getContract({
      address: config.bvsAddress,
      abi: bvs.abi,
      client: this.client,
    });
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
      const currentHeight = await this.client.getBlockNumber({ cacheTime: 0 });

      if (progress.height < currentHeight) {
        console.log(`[${this.label}] Processing blocks from height ${progress.height} to ${currentHeight}`);
        // Process all blocks from height to currentHeight
        for (let h = progress.height; h <= currentHeight && this.running; h++) {
          const events = await this.client.getContractEvents({
            address: this.bvsAddress,
            abi: bvs.abi,
            eventName: "PacketBroadcasted",
            fromBlock: BigInt(h),
            toBlock: BigInt(h),
          });

          for (const event of events) {
            // Find the input attribute
            const packet_guid = event.args.guid as `0x${string}`;
            if (packet_guid) {
              const packetRes = await this.bvsContract.read.getPacket([packet_guid]);

              // get payloadhash
              // @ts-expect-error packetRes is not typed in viem
              const payload_hash = this.verifyPacket(packet_guid, packetRes.packet.payload as `0x${string}`);
              // submit the packet to BVS contract
              await this.submitPacket(packet_guid, payload_hash);
              // log the result
              console.log(`[${this.label}] Submit packet: ${packet_guid}`);
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
   * Verify the packet by calculating the payload hash.
   * @param guid - The GUID of the packet.
   * @param message - The message of the packet.
   * @private
   */
  private verifyPacket(guid: `0x${string}`, message: `0x${string}`): `0x${string}` {
    // ** Implement your packet verification logic here **

    // For this case, we will do a keccak256 hash (guid, message)
    return keccak256(encodePacked(["bytes32", "bytes"], [guid, message]));
  }

  private async submitPacket(guid: `0x${string}`, payload_hash: `0x${string}`) {
    return this.bvsContract.write.submitPacket([guid, payload_hash], { account: this.operatorAddress });
  }
}
