import { QueryMsg as RouterQueryMsg } from "@satlayer/cosmwasm-schema/vault-router";
import { afterEach, beforeEach, expect, test } from "vitest";

import { CosmWasmContracts } from "./cosmwasm-contracts";
import { CosmWasmContainer, StartedCosmWasmContainer } from "./wasmd-container";

let started: StartedCosmWasmContainer;
let contracts: CosmWasmContracts;

beforeEach(async () => {
  started = await new CosmWasmContainer().start();
  contracts = await CosmWasmContracts.bootstrap(started);
}, 30_000);

afterEach(async () => {
  await started.stop();
});

test("should bootstrap contracts", async () => {
  const queryMsg: RouterQueryMsg = {
    list_vaults: {},
  };

  const client = contracts.client;
  const vaults = await client.queryContractSmart(contracts.data.router.address, queryMsg);
  expect(vaults).toEqual([]);
}, 30_000);

test("should deploy vaults", async () => {
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

  const queryMsg: RouterQueryMsg = {
    list_vaults: {},
  };

  const vaults = await contracts.router.query(queryMsg);
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

test("should deploy tokenized vaults", async () => {
  const operator = (await started.wallet.getAccounts())[6];

  const bankVault = await contracts.initVaultBankTokenized(operator.address, "ucosm");

  const cw20_address = await contracts.initCw20({
    decimals: 8,
    name: "JUMP",
    symbol: "JUMP",
    initial_balances: [
      {
        address: operator.address,
        amount: "1000000000000",
      },
    ],
  });
  const cw20Vault = await contracts.initVaultCw20Tokenized(operator.address, cw20_address);

  const queryMsg: RouterQueryMsg = {
    list_vaults: {},
  };

  const vaults = await contracts.router.query(queryMsg);
  expect(vaults).toEqual(
    expect.arrayContaining([
      {
        vault: bankVault,
        whitelisted: true,
      },
      {
        vault: cw20Vault,
        whitelisted: true,
      },
    ]),
  );
}, 30_000);
