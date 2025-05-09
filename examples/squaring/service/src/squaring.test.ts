import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { SatLayerContainer, StartedSatLayerContainer } from "@satlayer/testcontainers";
import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

describe("Squaring Node", { timeout: 120_000 }, () => {
  let container: StartedSatLayerContainer;

  beforeAll(async () => {
    container = await new SatLayerContainer().start();
  });

  afterAll(async () => {
    await container.stop();
  });

  test("should connect with height", async () => {
    const client = await CosmWasmClient.connect(container.getHostRpcUrl());
    const height = await client.getHeight();
    expect(height).toStrictEqual(expect.any(Number));
  });

  test("should start node and square", async () => {
    const wallet = await DirectSecp256k1HdWallet.generate(12, { prefix: "stake" });
    const [operator] = await wallet.getAccounts();
    const client = await SigningCosmWasmClient.connectWithSigner(container.getHostRpcUrl(), wallet);
    // TODO(fuxingloh):
  });
});
