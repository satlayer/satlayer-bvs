import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { stringToPath } from "@cosmjs/crypto";
import { coins, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { QueryMsg as RouterQueryMsg } from "@satlayer/cosmwasm-schema/vault-router";
import { afterAll, beforeAll, describe, expect, test } from "vitest";

import { SatLayerContainer, StartedSatLayerContainer } from "./container";

describe("SatLayerContainer", () => {
  let container: StartedSatLayerContainer;

  beforeAll(async () => {
    container = await new SatLayerContainer().start();
  }, 60_000);

  afterAll(async () => {
    await container.stop();
  });

  test("should getHeight", async () => {
    const client = await CosmWasmClient.connect(container.getHostRpcUrl());

    expect(await client.getHeight()).toStrictEqual(expect.any(Number));
  });

  test("should getChainId", async () => {
    const client = await CosmWasmClient.connect(container.getHostRpcUrl());

    expect(await client.getChainId()).toStrictEqual("wasm-1337");
  });

  test("should fund address and transfer from funded address", async () => {
    const mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon cactus";
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "wasm",
      hdPaths: [stringToPath("m/44'/118'/1'/0/20"), stringToPath("m/44'/118'/1'/0/21")],
    });

    const [jessica, elizabeth] = await wallet.getAccounts();
    const tx = await container.fund(jessica.address, "100000ustake");
    expect(tx.events).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          type: "transfer",
        }),
      ]),
    );

    const signer = await SigningCosmWasmClient.connectWithSigner(container.getHostRpcUrl(), wallet, {
      gasPrice: GasPrice.fromString("0.002000ustake"),
    });
    const balance = await signer.getBalance(jessica.address, "ustake");
    expect(balance).toEqual({
      amount: "100000",
      denom: "ustake",
    });

    const tx2 = await signer.sendTokens(jessica.address, elizabeth.address, coins(1, "ustake"), "auto");
    await container.waitForTx(tx2.transactionHash);

    const balance2 = await signer.getBalance(elizabeth.address, "ustake");
    expect(balance2).toEqual({
      amount: "1",
      denom: "ustake",
    });
  }, 30_000);
});

test("should bootstrap contracts", async () => {
  const started = await new SatLayerContainer().start();
  const contracts = await started.bootstrap();

  const client = await CosmWasmClient.connect(started.getHostRpcUrl());
  const queryMsg: RouterQueryMsg = {
    list_vaults: {},
  };

  const vaults: any = await client.queryContractSmart(contracts.router, queryMsg);

  expect(vaults).toEqual(
    expect.arrayContaining([
      {
        vault: contracts.vaults.bank,
        whitelisted: true,
      },
      {
        vault: contracts.vaults.cw20,
        whitelisted: true,
      },
    ]),
  );
}, 180_000);
