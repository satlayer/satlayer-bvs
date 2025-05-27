import { afterAll, beforeAll, expect, test, vi } from "vitest";
import { CosmWasmContainer, StartedCosmWasmContainer, SatLayerContracts } from "@satlayer/testcontainers";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { readFile } from "node:fs/promises";
import { stringToPath } from "@cosmjs/crypto";
import { ExecuteMsg, InstantiateMsg } from "./types";

let started: StartedCosmWasmContainer;
let contracts: SatLayerContracts;
let wallet: DirectSecp256k1HdWallet;
let client: SigningCosmWasmClient;
let governanceContractAddress: string;

async function deployContract(owner: string) {
  const contractPath = require.resolve("@examples/governance-contract/dist/contract.wasm");
  const uploaded = await client.upload(owner, await readFile(contractPath), "auto");
  const initMsg: InstantiateMsg = {
    registry: contracts.registry.address,
    router: contracts.router.address,
    owner: owner,
  };
  return client.instantiate(owner, uploaded.codeId, initMsg, "governance", "auto");
}

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
  governanceContractAddress = instantiated.contractAddress;

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

  // // Create the Squaring and Service nodes
  // squaringNode = new SquaringNode(client, instantiated.contractAddress, operator.address);
  //
  // // Start the squaring node without awaiting = this is async.
  // void squaringNode.start(0);
}, 60_000);

test("Hello World", async () => {
  await new Promise((resolve) => setTimeout(resolve, 5_000));
}, 15_000);
