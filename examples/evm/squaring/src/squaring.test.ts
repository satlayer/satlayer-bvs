import { AnvilContainer, ChainName, EVMContracts, StartedAnvilContainer } from "@satlayer/testcontainers";
import bvs from "./contracts/out/BVS.sol/BVS.json";
import { encodeFunctionData, getContract, pad, padHex, parseEther } from "viem";
import { beforeAll, expect, test, vi } from "vitest";

let ethNodeStarted: StartedAnvilContainer;
let opNodeStarted: StartedAnvilContainer;
let contracts: EVMContracts;

let ownerAddress: `0x${string}`;
let operatorAddress: `0x${string}`;
let bvsContract: Awaited<ReturnType<typeof ethNodeStarted.deployContract>>;

async function initBVS(ethNodeStarted: StartedAnvilContainer): Promise<void> {
  bvsContract = await ethNodeStarted.deployContract({
    abi: bvs.abi,
    bytecode: bvs.bytecode.object as unknown as `0x${string}`,
    salt: "bvs",
    constructorArgs: [contracts.registry.address, contracts.router.address],
  });

  await bvsContract.contract.write.enableSlashing([
    {
      destination: bvsContract.contractAddress,
      maxMbips: 100_000,
      resolutionWindow: 864000, // 10 days;
    },
  ]);

  // register operators to the BVS contract
  await contracts.registry.write.registerAsOperator(["www.operator.com", "operator"], { account: operatorAddress });

  await contracts.registry.write.registerServiceToOperator([bvsContract.contractAddress], {
    account: operatorAddress,
  });
}

beforeAll(async () => {
  // run local forked nodes
  let ethNode = new AnvilContainer({
    forkedChainName: ChainName.EthereumMainnet,
  });

  ethNodeStarted = await ethNode.start();

  // setup local cosmos node with satlayer contracts
  contracts = await EVMContracts.bootstrap(ethNodeStarted);

  const owner = ethNodeStarted.generateAccount("Owner");
  const operator = ethNodeStarted.generateAccount("operator1");

  ownerAddress = owner.address;
  operatorAddress = operator.address;

  // Fund all accounts with some gas
  await ethNodeStarted.setBalance(ownerAddress, parseEther("1"));
  await ethNodeStarted.setBalance(operatorAddress, parseEther("1"));

  await ethNodeStarted.mineBlock(1);

  // Initializes the BVS contract and registers it with the registry and operators.
  await initBVS(ethNodeStarted);
  await ethNodeStarted.mineBlock(1);
}, 120_000);

test("should compute off-chain and get response on-chain", async () => {}, 15_000);
