import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { afterAll, beforeAll, expect, test } from "vitest";

import { CosmWasmContainer, StartedCosmWasmContainer } from "./container";

let started: StartedCosmWasmContainer;

beforeAll(async () => {
  started = await new CosmWasmContainer().start();
}, 30_000);

afterAll(async () => {
  await started.stop();
});

test("should getHeight", async () => {
  const client = await CosmWasmClient.connect(started.getRpcEndpoint());

  expect(await client.getHeight()).toStrictEqual(expect.any(Number));
});

test("should getChainId", async () => {
  const client = await CosmWasmClient.connect(started.getRpcEndpoint());

  expect(await client.getChainId()).toStrictEqual("wasm-1337");
});
