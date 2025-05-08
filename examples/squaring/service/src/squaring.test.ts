import { afterAll, beforeAll, describe, expect, test, vi } from "vitest";
import { CosmWasmContainer, StartedCosmWasmContainer, SatLayerContracts } from "@satlayer/testcontainers";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { SquaringNode, ServiceNode } from "./squaring";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFile } from "node:fs/promises";
import { stringToPath } from "@cosmjs/crypto";
import { InstantiateMsg } from "./contract";

describe("Squaring and Service node non-fault", () => {
  let started: StartedCosmWasmContainer;
  let contracts: SatLayerContracts;
  let wallet: DirectSecp256k1HdWallet;
  let client: SigningCosmWasmClient;

  let squaringNode: SquaringNode;
  let serviceNode: ServiceNode;

  beforeAll(async () => {
    started = await new CosmWasmContainer().start();
    contracts = await SatLayerContracts.bootstrap(started);

    wallet = await DirectSecp256k1HdWallet.generate(12, {
      prefix: "wasm",
      hdPaths: [stringToPath("m/0"), stringToPath("m/1"), stringToPath("m/2")],
    });

    client = await SigningCosmWasmClient.connectWithSigner(started.getRpcEndpoint(), wallet, {
      gasPrice: GasPrice.fromString("0.002000ustake"),
      broadcastPollIntervalMs: 200,
    });

    // Fund all accounts with some tokens
    for (const account of await wallet.getAccounts()) {
      await started.fund(account.address, "10000000ustake");
    }

    const [{ address: ownerAddr }, { address: operatorAddr }] = await wallet.getAccounts();
    const contractPath = require.resolve("@examples/squaring-contract/dist/contract.wasm");
    const uploaded = await client.upload(ownerAddr, await readFile(contractPath), "auto");
    const initMsg: InstantiateMsg = {
      registry: contracts.registry,
      router: contracts.router,
      owner: ownerAddr,
    };
    const instantiated = await client.instantiate(ownerAddr, uploaded.codeId, initMsg, "Squaring", "auto");

    await client.execute(
      operatorAddr,
      contracts.registry,
      {
        register_as_operator: {
          metadata: {},
        },
      },
      "auto",
    );

    await client.execute(
      operatorAddr,
      contracts.registry,
      {
        register_service_to_operator: {
          service: instantiated.contractAddress,
        },
      },
      "auto",
    );

    await client.execute(
      ownerAddr,
      instantiated.contractAddress,
      {
        register_operator: {
          operator: operatorAddr,
        },
      },
      "auto",
    );

    squaringNode = new SquaringNode(client, instantiated.contractAddress, operatorAddr);
    serviceNode = new ServiceNode(client, instantiated.contractAddress, operatorAddr);

    // Start the squaring node without awaiting = this is async.
    void squaringNode.start(0);
  }, 60_000);

  afterAll(async () => {
    await started.stop();
    await squaringNode.stop();
  });

  test("should compute off-chain", async () => {
    const sender = await wallet.getAccounts().then((accounts) => accounts[2].address);
    await serviceNode.request(sender, 99);

    await vi.waitFor(async () => {
      const response = await serviceNode.getResponse(99);
      expect(response).toEqual(9801);
    }, 10_000);
  }, 15_000);
});
