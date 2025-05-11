import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { SatLayerContainer, StartedSatLayerContainer } from "@satlayer/testcontainers";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

describe("Squaring", () => {
  let container: StartedSatLayerContainer;

  beforeAll(async () => {
    container = await new SatLayerContainer().start();
  }, 60_000);

  afterAll(async () => {
    await container.stop();
  });

  test("should connect with height", async () => {
    const rpcUrl = container.getHostRpcUrl();
    const client = await CosmWasmClient.connect(rpcUrl);
    const height = await client.getHeight();
    expect(height).toStrictEqual(expect.any(Number));
  });
});
