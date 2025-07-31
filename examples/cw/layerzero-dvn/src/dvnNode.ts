import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { setTimeout } from "node:timers/promises";
import { ChainName, SuperTestClient } from "@satlayer/testcontainers";

import endpointV2 from "@layerzerolabs/lz-evm-protocol-v2/artifacts/contracts/EndpointV2.sol/EndpointV2.json";
import customDVN from "./evm/out/CustomDVN.sol/CustomDVN.json";
import { LZ_CONTRACTS } from "./lz.constant";
import { encodePacked, keccak256, Log } from "viem";
import { PacketV1Codec, trim0x } from "@layerzerolabs/lz-v2-utilities";
import { ExecuteMsg, GetPacketResponse, QueryMsg } from "./bvs-dvn-contract";

/**
 * DVNNode is a node that watches the src chain (eth mainnet) for packets assigned.
 */
export class DVNNode {
  private running: boolean = true;

  private readonly cosmosClient: SigningCosmWasmClient;
  private readonly ethClient: SuperTestClient;
  private readonly opClient: SuperTestClient;

  private readonly ownerAddress: string;
  private readonly bvsAddress: string;
  private readonly dvnEthContractAddress: `0x${string}`;

  constructor(config: {
    cosmosClient: SigningCosmWasmClient;
    ethClient: SuperTestClient;
    opClient: SuperTestClient;
    ownerAddress: string;
    bvsAddress: string;
    dvnEthContractAddress: `0x${string}`;
  }) {
    this.cosmosClient = config.cosmosClient;
    this.ethClient = config.ethClient;
    this.opClient = config.opClient;
    this.ownerAddress = config.ownerAddress;
    this.bvsAddress = config.bvsAddress;
    this.dvnEthContractAddress = config.dvnEthContractAddress;
  }

  /**
   * Start the DVN node that watches the src chain (eth mainnet) for packets assigned.
   * It will first process all blocks from startFrom height and then continue to listen for new blocks.
   *
   * It will look for:
   *  - `PacketSent` events in LZ endpointV2 contract (eth mainnet)
   *  - `PacketAssigned` events in DVNApp contract (eth mainnet)
   *  - For each `PacketSent` event, it will check if it is assigned to DVNApp contract, if so, it will broadcast the packet to BVS for operators to validate.
   * @param startFrom
   */
  public async startEthNode(startFrom: bigint) {
    let progress = { height: startFrom };

    while (this.running) {
      const currentHeight = await this.ethClient.getBlockNumber({ cacheTime: 0 });

      if (progress.height <= currentHeight) {
        // Process all blocks from height to currentHeight
        for (let h = progress.height; h <= currentHeight && this.running; h++) {
          // get all PacketSent events in endpointV2 contract (eth mainnet) at a given block height
          const packetSentEvents = await this.getPacketSentEvent(h);
          if (packetSentEvents.length > 0) {
            // get all PacketAssigned events in dvnapp contract (eth mainnet) at a given block height
            const packetAssignedEvents = await this.getPacketAssignedEvent(h);
            // convert packetAssignedEvents to a Set of packet hashes for quick lookup
            const packetAssignedHashSet = new Set(packetAssignedEvents.map((event) => event.args.packetHash));

            // for each PacketSent event, check if it is assigned to DVNApp contract
            for (const packetSentEvent of packetSentEvents) {
              const packet = PacketV1Codec.from(packetSentEvent.args.encodedPayload);
              let packetHash = keccak256(
                encodePacked(
                  ["bytes", "bytes32"],
                  [packet.header() as `0x${string}`, packet.payloadHash() as `0x${string}`],
                ),
              );
              if (packetAssignedHashSet.has(packetHash)) {
                // TODO: do idempotency check on DVN contract to see if packet is already verified

                // broadcast packet to BVS for operators to validate
                await this.broadcastPacketToBVS(packet);
                console.log(`[DVNNode] Broadcasted packet ${packet.guid()} to BVS for validation`);
              }
            }
          }
          // Update progress after processing each height
          progress.height = h + BigInt(1);
        }
      } else {
        // No new blocks, sleep for 1 second
        await setTimeout(1000);
      }
    }
  }

  public async finalizedPacket(packet_guid: string) {
    let finalizePacketMsg: ExecuteMsg = {
      finalize_packet_verification: {
        packet_guid,
      },
    };
    return this.cosmosClient.execute(this.ownerAddress, this.bvsAddress, finalizePacketMsg, "auto");
  }

  public async getFinalizedPayloadHash(packet_guid: string): Promise<string | undefined> {
    let getPacket: QueryMsg = {
      get_packet: packet_guid,
    };
    const response: GetPacketResponse = await this.cosmosClient.queryContractSmart(this.bvsAddress, getPacket);
    return response.packet_verification_finalized?.payload_hash;
  }

  /**
   * Get all PacketSent events in endpointV2 contract (eth mainnet) at a given block height.
   * @param height
   * @private
   */
  private async getPacketSentEvent(height: bigint): Promise<Log[]> {
    return this.ethClient.getContractEvents({
      abi: endpointV2.abi,
      eventName: "PacketSent",
      address: LZ_CONTRACTS[ChainName.EthereumMainnet].endpointV2,
      fromBlock: height,
      toBlock: height,
    });
  }

  /**
   * Get all PacketAssigned events in dvnapp contract (eth mainnet) at a given block height.
   * @param height
   * @private
   */
  private async getPacketAssignedEvent(height: bigint): Promise<Log[]> {
    return this.ethClient.getContractEvents({
      abi: customDVN.abi,
      eventName: "PacketAssigned",
      address: this.dvnEthContractAddress,
      fromBlock: height,
      toBlock: height,
    });
  }

  /**
   * Broadcast a packet to BVS for operators to validate.
   * It will execute a `broadcast_packet` message to the BVS contract (cosmos chain)
   * @param packet
   */
  private async broadcastPacketToBVS(packet: PacketV1Codec) {
    let broadcastPacketMsg: ExecuteMsg = {
      broadcast_packet: {
        dst_eid: packet.dstEid(),
        guid: trim0x(packet.guid()),
        message: trim0x(packet.message()),
        nonce: packet.nonce(),
        receiver: trim0x(packet.receiver()),
        sender: trim0x(packet.sender()),
        src_eid: packet.srcEid(),
      },
    };
    return await this.cosmosClient.execute(this.ownerAddress, this.bvsAddress, broadcastPacketMsg, "auto");
  }
}
