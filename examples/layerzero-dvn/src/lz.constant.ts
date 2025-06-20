import { ChainName } from "@satlayer/testcontainers";

export interface LZContracts {
  eid: number;
  endpointV2: `0x${string}`;
  sendUln302: `0x${string}`;
  receiveUln302: `0x${string}`;
  ReadLib1002: `0x${string}`;
  BlockedMessageLib: `0x${string}`;
  LZExecutor: `0x${string}`;
}

/**
 * Stores the LayerZero contracts addresses for each chain.
 */
export const LZ_CONTRACTS: Record<ChainName, LZContracts> = {
  [ChainName.EthereumMainnet]: {
    eid: 30101,
    endpointV2: "0x1a44076050125825900e736c501f859c50fE728c",
    sendUln302: "0xbB2Ea70C9E858123480642Cf96acbcCE1372dCe1",
    receiveUln302: "0xc02Ab410f0734EFa3F14628780e6e695156024C2",
    ReadLib1002: "0x74F55Bc2a79A27A0bF1D1A35dB5d0Fc36b9FDB9D",
    BlockedMessageLib: "0x1ccbf0db9c192d969de57e25b3ff09a25bb1d862",
    LZExecutor: "0x173272739Bd7Aa6e4e214714048a9fE699453059",
  },
  [ChainName.OptimismMainnet]: {
    eid: 30111,
    endpointV2: "0x1a44076050125825900e736c501f859c50fE728c",
    sendUln302: "0x1322871e4ab09Bc7f5717189434f97bBD9546e95",
    receiveUln302: "0x3c4962Ff6258dcfCafD23a814237B7d6Eb712063",
    ReadLib1002: "0x01B29c03fAD8F455184573D6624a8136cF6106Fb",
    BlockedMessageLib: "0x1ccbf0db9c192d969de57e25b3ff09a25bb1d862",
    LZExecutor: "0x2D2ea0697bdbede3F01553D2Ae4B8d0c486B666e",
  },
};
