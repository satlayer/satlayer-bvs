import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { CosmWasmContainer, StartedCosmWasmContainer } from "@satlayer/testcontainers";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

describe("Squaring", () => {
  let started: StartedCosmWasmContainer;

  beforeAll(async () => {
    started = await new CosmWasmContainer().start();
  }, 60_000);

  afterAll(async () => {
    await started.stop();
  });

  test("should connect with height", async () => {
    const rpcUrl = started.getRpcEndpoint();
    const client = await CosmWasmClient.connect(rpcUrl);
    const height = await client.getHeight();
    expect(height).toStrictEqual(expect.any(Number));
  });
});
