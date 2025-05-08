import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { afterAll, beforeAll, describe, expect, test } from "vitest";

import { SatLayerContainer, StartedSatLayerContainer } from "./satlayer";

describe("SatLayerContainer", { timeout: 60_000 }, () => {
  let container: StartedSatLayerContainer;

  beforeAll(async () => {
    container = await new SatLayerContainer().start();
  });

  afterAll(async () => {
    await container.stop();
  });

  test("should start and connect", async () => {
    const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon cactus";
    const signer = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "bbn",
    });

    const rpcUrl = container.getHostRpcUrl();
    const gasPrice = GasPrice.fromString("0.002000stake");
    const client = await SigningCosmWasmClient.connectWithSigner(rpcUrl, signer, { gasPrice });
    const height = await client.getHeight();
    expect(height).toStrictEqual(expect.any(Number));
  });
});

// const chainId = "bbn-test-5";
// const endpoint = "https://babylon-testnet-rpc.nodes.guru";
//
// import { GasPrice } from "@cosmjs/stargate";
//
// const gasPrice = GasPrice.fromString("0.002000ubbn");
//
// import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
// import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
// import { stringToPath } from "@cosmjs/crypto";
//
// const signer = await DirectSecp256k1HdWallet.fromMnemonic(secret.stdout.trim(), {
//   prefix: "bbn",
// });
// const client = await SigningCosmWasmClient.connectWithSigner(endpoint, signer, { gasPrice });
//

//
// const [deployer, operator] = await signer.getAccounts();
