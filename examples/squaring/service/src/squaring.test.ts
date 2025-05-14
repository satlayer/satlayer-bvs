import { afterAll, beforeAll, expect, test, vi } from "vitest";
import { CosmWasmContainer, StartedCosmWasmContainer, SatLayerContracts } from "@satlayer/testcontainers";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { SquaringNode, ServiceNode } from "./squaring";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFile } from "node:fs/promises";
import { stringToPath } from "@cosmjs/crypto";
import { InstantiateMsg } from "./contract";

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

  // Fund all 3 accounts with some tokens
  for (const account of await wallet.getAccounts()) {
    await started.fund(account.address, "10000000ustake");
  }

  client = await SigningCosmWasmClient.connectWithSigner(started.getRpcEndpoint(), wallet, {
    gasPrice: GasPrice.fromString("0.002000ustake"),
    broadcastPollIntervalMs: 200,
  });

  const [owner, operator] = await wallet.getAccounts();
  const contractPath = require.resolve("@examples/squaring-contract/dist/contract.wasm");
  const uploaded = await client.upload(owner.address, await readFile(contractPath), "auto");
  const initMsg: InstantiateMsg = {
    registry: contracts.registry.address,
    router: contracts.router.address,
    owner: owner.address,
  };
  const instantiated = await client.instantiate(owner.address, uploaded.codeId, initMsg, "Squaring", "auto");

  await contracts.registry.execute(operator.address, {
    register_as_operator: {
      metadata: {},
    },
  });
  await contracts.registry.execute(operator.address, {
    register_service_to_operator: {
      service: instantiated.contractAddress,
    },
  });

  await client.execute(
    owner.address,
    instantiated.contractAddress,
    {
      register_operator: {
        operator: operator.address,
      },
    },
    "auto",
  );

  squaringNode = new SquaringNode(client, instantiated.contractAddress, operator.address);
  serviceNode = new ServiceNode(client, instantiated.contractAddress, operator.address);

  // Start the squaring node without awaiting = this is async.
  void squaringNode.start(0);
}, 60_000);

afterAll(async () => {
  await started.stop();
  await squaringNode.stop();
});

test("should compute off-chain and get response on-chain", async () => {
  const sender = await wallet.getAccounts().then((accounts) => accounts[2].address);
  await serviceNode.request(sender, 99);

  await vi.waitFor(async () => {
    const response = await serviceNode.getResponse(99);
    expect(response).toEqual(9801);
  }, 10_000);
}, 15_000);
