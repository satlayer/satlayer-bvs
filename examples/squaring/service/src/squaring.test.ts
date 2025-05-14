import { afterAll, beforeAll, expect, test, vi } from "vitest";
import { CosmWasmContainer, StartedCosmWasmContainer, SatLayerContracts } from "@satlayer/testcontainers";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { SquaringNode, ServiceNode } from "./squaring";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { readFile } from "node:fs/promises";
import { stringToPath } from "@cosmjs/crypto";
import { ExecuteMsg, InstantiateMsg } from "./contract";

let started: StartedCosmWasmContainer;
let contracts: SatLayerContracts;
let wallet: DirectSecp256k1HdWallet;
let client: SigningCosmWasmClient;

let squaringNode: SquaringNode;
let serviceNode: ServiceNode;

beforeAll(async () => {
  // Set up CosmWasmContainer with SatLayerContracts bootstrapped
  started = await new CosmWasmContainer().start();
  contracts = await SatLayerContracts.bootstrap(started);

  // A wallet with 3 accounts, owner of contract, operator, and anyone (for testing)
  wallet = await DirectSecp256k1HdWallet.generate(12, {
    prefix: "wasm",
    hdPaths: [stringToPath("m/0"), stringToPath("m/1"), stringToPath("m/2")],
  });
  const [owner, operator, anyone] = await wallet.getAccounts();

  // Fund all 3 accounts with some tokens
  await started.fund("10000000ustake", owner.address, operator.address, anyone.address);

  // Create a new signer with the wallet using the container as the RPC endpoint
  client = await started.newSigner(wallet);

  // Deploy Squaring Contract
  const instantiated = await deployContract(owner.address);

  // Register the operator and the service to the operator
  await contracts.registry.execute(client, operator.address, {
    register_as_operator: {
      metadata: {},
    },
  });
  await contracts.registry.execute(client, operator.address, {
    register_service_to_operator: {
      service: instantiated.contractAddress,
    },
  });
  const registerOperator: ExecuteMsg = {
    register_operator: {
      operator: operator.address,
    },
  };
  await client.execute(owner.address, instantiated.contractAddress, registerOperator, "auto");

  // Create the Squaring and Service nodes
  squaringNode = new SquaringNode(client, instantiated.contractAddress, operator.address);
  serviceNode = new ServiceNode(client, instantiated.contractAddress, operator.address);

  // Start the squaring node without awaiting = this is async.
  void squaringNode.start(0);
}, 60_000);

afterAll(async () => {
  await started.stop();
  await squaringNode.stop();
});

async function deployContract(owner: string) {
  const contractPath = require.resolve("@examples/squaring-contract/dist/contract.wasm");
  const uploaded = await client.upload(owner, await readFile(contractPath), "auto");
  const initMsg: InstantiateMsg = {
    registry: contracts.registry.address,
    router: contracts.router.address,
    owner: owner,
  };
  return client.instantiate(owner, uploaded.codeId, initMsg, "Squaring", "auto");
}

test("should compute off-chain and get response on-chain", async () => {
  const anyone = await wallet.getAccounts().then((accounts) => accounts[2].address);

  // Request the squaring node to compute the square of 99
  await serviceNode.request(anyone, 99);

  // Wait for the squaring node to compute the square and store the result on-chain
  await vi.waitFor(async () => {
    const response = await serviceNode.getResponse(99);
    expect(response).toEqual(9801);
  }, 10_000);
}, 15_000);
