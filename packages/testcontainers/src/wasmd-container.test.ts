import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { stringToPath } from "@cosmjs/crypto";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { afterAll, beforeAll, expect, test } from "vitest";

import { CosmWasmContainer, StartedCosmWasmContainer } from "./wasmd-container";

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

test("should fund addresses", async () => {
  const wallet = await DirectSecp256k1HdWallet.generate(12, {
    prefix: "wasm",
    hdPaths: [stringToPath("m/0"), stringToPath("m/1"), stringToPath("m/2")],
  });
  const [alice, angler, angel] = await wallet.getAccounts();
  await started.fund("1000ustake", alice.address, angler.address, angel.address);

  expect(await started.client.getBalance(alice.address, "ustake")).toEqual({
    denom: "ustake",
    amount: "1000",
  });

  expect(await started.client.getBalance(angler.address, "ustake")).toEqual({
    denom: "ustake",
    amount: "1000",
  });

  expect(await started.client.getBalance(angel.address, "ustake")).toEqual({
    denom: "ustake",
    amount: "1000",
  });
});
