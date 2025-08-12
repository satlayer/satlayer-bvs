import {
  AnvilContainer,
  ChainName,
  EVMContracts,
  StartedAnvilContainer,
  SuperTestClient,
} from "@satlayer/testcontainers";
import { abi, bytecode } from "./contracts/out/BVS.sol/BVS.json";
import { Account, encodeFunctionData, getContract, pad, padHex, parseEther } from "viem";
import { afterAll, beforeAll, expect, test, vi } from "vitest";
import { randomBytes } from "node:crypto";
import { ServiceNode, SquaringNode } from "./squaring";

let ethNodeStarted: StartedAnvilContainer;
let contracts: EVMContracts;

let owner: Account;
let operator: Account;
let anyone: Account;
let bvsContract: Awaited<ReturnType<typeof ethNodeStarted.deployContract>>;
let squaringNode: SquaringNode;

async function initBVS(ethNodeStarted: StartedAnvilContainer): Promise<void> {
  bvsContract = await ethNodeStarted.deployContract({
    abi: abi,
    salt: "squaring-contract",
    bytecode: bytecode.object as unknown as `0x${string}`,
    constructorArgs: [contracts.router.address, contracts.registry.address, owner.address],
  });

  await ethNodeStarted.mineBlock(1);

  await bvsContract.contract.write.enableSlashing([
    {
      destination: bvsContract.contractAddress,
      maxMbips: 100_000,
      resolutionWindow: 864000, // 10 days;
    },
  ]);

  await ethNodeStarted.mineBlock(1);

  await contracts.registry.write.registerAsOperator(["www.SquaringOperator.com", "SquaringOperator"], {
    account: operator.address,
  });

  await ethNodeStarted.mineBlock(2);

  // register operators to the BVS contract
  await contracts.registry.write.registerServiceToOperator([bvsContract.contractAddress], {
    account: operator.address,
  });

  await ethNodeStarted.mineBlock(2);
  await bvsContract.contract.write.registerOperator([operator.address], {
    account: owner.address,
  });
  await ethNodeStarted.mineBlock(1);
}

beforeAll(async () => {
  // run local forked nodes
  let ethNode = new AnvilContainer({
    forkedChainName: ChainName.EthereumMainnet,
  });

  ethNodeStarted = await ethNode.start();

  // setup local cosmos node with satlayer contracts
  contracts = await EVMContracts.bootstrap(ethNodeStarted);

  owner = ethNodeStarted.getAccount() as unknown as Account;
  operator = ethNodeStarted.generateAccount("operator1") as unknown as Account;
  anyone = ethNodeStarted.generateAccount("anyone") as unknown as Account;

  // Fund all accounts with some gas
  await ethNodeStarted.setBalance(owner.address, parseEther("1"));
  await ethNodeStarted.setBalance(operator.address, parseEther("1"));
  await ethNodeStarted.setBalance(anyone.address, parseEther("1"));

  await ethNodeStarted.mineBlock(1);

  // Initializes the BVS contract and registers it with the registry and operators.
  await initBVS(ethNodeStarted);
  await ethNodeStarted.mineBlock(1);
  squaringNode = new SquaringNode(
    ethNodeStarted.getClient(),
    ethNodeStarted,
    operator,
    getContract({
      address: bvsContract.contractAddress,
      abi: abi,
      client: ethNodeStarted.getClient(),
    }),
  );

  void squaringNode.start(BigInt(5));
}, 120_000);

afterAll(async () => {
  await ethNodeStarted.stop();
  await squaringNode.stop();
});

test("should compute off-chain and get response on-chain", async () => {
  const service = new ServiceNode(
    getContract({
      address: bvsContract.contractAddress,
      abi: abi,
      client: ethNodeStarted.getClient(),
    }),
  );

  // Request the squaring node to compute the square of 99
  await service.request(anyone.address, 99);
  await ethNodeStarted.mineBlock(1);

  // Wait for the squaring node to compute the square and store the result on-chain
  await vi.waitFor(async () => {
    const response = await service.getResponse(99, operator.address);
    expect(response).toEqual(9801n);
  }, 20_000);
}, 25_000);
