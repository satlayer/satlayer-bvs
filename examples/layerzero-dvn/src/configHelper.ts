import { encodeAbiParameters } from "viem";

// https://github.com/LayerZero-Labs/LayerZero-v2/blob/88428755be6caa71cb1d2926141d73c8989296b5/packages/layerzero-v2/evm/messagelib/contracts/uln/uln302/SendUln302.sol#L16-L17
export enum ConfigType {
  EXECUTOR = 1,
  ULN = 2,
}

// https://github.com/LayerZero-Labs/LayerZero-v2/blob/88428755be6caa71cb1d2926141d73c8989296b5/packages/layerzero-v2/evm/messagelib/contracts/uln/UlnBase.sol#L8-L16
export interface UlnConfig {
  // The number of block confirmations to wait before emitting message on the destination chain.
  // null means no confirmations are required, 0 means default confirmations are used.
  confirmations: bigint | null;
  // The address of the DVNs you will pay to verify a sent message on the source chain.
  // The destination tx will wait until ALL `requiredDVNs` verify the message.
  // `null` means no DVNs are required, empty array means default DVNs are used instead.
  requiredDVNs: string[] | null;
  // The address of the DVNs you will pay to verify a sent message on the source chain.
  // The destination tx will wait until the configured threshold of `optionalDVNs` verify a message.
  // `null` means no DVNs are required, empty array means default DVNs are used instead.
  optionalDVNs: string[] | null;
  // The number of `optionalDVNs` that need to successfully verify the message for it to be considered Verified.
  optionalDVNThreshold: number;
}

export function encodeUlnConfig(config: UlnConfig): string {
  return encodeAbiParameters(
    [
      { name: "confirmations", type: "uint64" },
      { name: "requiredDVNCount", type: "uint8" },
      { name: "optionalDVNCount", type: "uint8" },
      { name: "optionalDVNThreshold", type: "uint8" },
      { name: "requiredDVNs", type: "address[]" },
      { name: "optionalDVNs", type: "address[]" },
    ],
    [
      config.confirmations ?? ConfigHelper.NIL_CONFIRMATIONS,
      config.requiredDVNs === null ? ConfigHelper.NIL_DVN_COUNT : config.requiredDVNs.length,
      config.optionalDVNs === null ? ConfigHelper.NIL_DVN_COUNT : config.optionalDVNs.length,
      config.optionalDVNThreshold,
      config.requiredDVNs == null ? [] : config.requiredDVNs.map((dvn) => dvn as `0x${string}`),
      config.optionalDVNs == null ? [] : config.optionalDVNs.map((dvn) => dvn as `0x${string}`),
    ],
  );
}

// https://github.com/LayerZero-Labs/LayerZero-v2/blob/88428755be6caa71cb1d2926141d73c8989296b5/packages/layerzero-v2/evm/messagelib/contracts/SendLibBase.sol#L23-L26
export interface ExecutorConfig {
  // The maximum size of the message that can be sent.
  maxMessageSize: number;
  // The address of the executor that will execute the message on the destination chain.
  executor: string;
}

export function encodeExecutorConfig(config: ExecutorConfig): string {
  return encodeAbiParameters(
    [
      { name: "maxMessageSize", type: "uint32" },
      { name: "executor", type: "address" },
    ],
    [config.maxMessageSize, config.executor as `0x${string}`],
  );
}

type Config = {
  eid: number;
} & ({ configType: ConfigType.EXECUTOR; config: ExecutorConfig } | { configType: ConfigType.ULN; config: UlnConfig });

/**
 * Helper class to convert configs to the format required by the LayerZero contracts.
 *
 * Config is used to configure Executor and ULN config
 */
export class ConfigHelper {
  configs: Config[] = [];

  /**
   * Values are from https://github.com/LayerZero-Labs/LayerZero-v2/blob/88428755be6caa71cb1d2926141d73c8989296b5/packages/layerzero-v2/evm/messagelib/contracts/uln/UlnBase.sol#L24
   */
  static NIL_DVN_COUNT = 255; // No DVNs required (uint8 max value)
  // TODO: fix as this doesnt seems to work with solidity uint64 max
  static NIL_CONFIRMATIONS = BigInt("18446744073709551615"); // No confirmations required (uint64 max value)

  protected constructor(configs: Config[]) {
    this.configs = configs;
  }

  static from(configs: Config[]): ConfigHelper {
    return new ConfigHelper(configs);
  }

  addConfig(config: Config): ConfigHelper {
    this.configs.push(config);
    return this;
  }

  toConfigParams(): any[][] {
    let configParams: any[][] = [];
    this.configs.forEach((c) => {
      let params = "";
      if (c.configType === ConfigType.EXECUTOR /* EXECUTOR */) {
        params = encodeExecutorConfig(c.config);
      } else if (c.configType === ConfigType.ULN /* VERIFIER */) {
        params = encodeUlnConfig(c.config);
      }
      configParams.push([c.eid, c.configType, params]);
    });
    return configParams;
  }
}
