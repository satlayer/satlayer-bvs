import { QueryMsg as RouterQueryMsg } from "@satlayer/cosmwasm-schema/vault-router";
import { afterEach, beforeEach, expect, test } from "vitest";

import { CosmWasmContainer, StartedCosmWasmContainer } from "./container";
import { SatLayerContracts } from "./satlayer";

let started: StartedCosmWasmContainer;
let contracts: SatLayerContracts;

beforeEach(async () => {
  started = await new CosmWasmContainer().start();
  contracts = await SatLayerContracts.bootstrap(started);
}, 30_000);

afterEach(async () => {
  await started.stop();
});

test("should bootstrap contracts", async () => {
  const client = contracts.client;
  const queryMsg: RouterQueryMsg = {
    list_vaults: {},
  };

  const vaults = await client.queryContractSmart(contracts.state.router.address, queryMsg);
  expect(vaults).toEqual([]);
}, 30_000);

test("should bootstrap contracts and deploy vaults", async () => {
  const operator = (await started.wallet.getAccounts())[5];

  const vault1 = await contracts.initVaultBank(operator.address, "ucosm");
  const vault2 = await contracts.initVaultBank(operator.address, "ustake");

  const cw20_address = await contracts.initCw20({
    decimals: 8,
    name: "BTC",
    symbol: "tBTC",
    initial_balances: [
      {
        address: operator.address,
        amount: "100000000000",
      },
    ],
  });
  const vault3 = await contracts.initVaultCw20(operator.address, cw20_address);

  const client = contracts.client;
  const queryMsg: RouterQueryMsg = {
    list_vaults: {},
  };

  const vaults = await client.queryContractSmart(contracts.state.router.address, queryMsg);
  expect(vaults).toEqual(
    expect.arrayContaining([
      {
        vault: vault1,
        whitelisted: true,
      },
      {
        vault: vault2,
        whitelisted: true,
      },
      {
        vault: vault3,
        whitelisted: true,
      },
    ]),
  );
}, 30_000);
