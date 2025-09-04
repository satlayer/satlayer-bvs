import { afterAll, beforeAll, expect, test, vi } from "vitest";
import {
  AnvilContainer,
  ChainName,
  EVMContracts,
  StartedAnvilContainer,
  SuperTestClient,
} from "@satlayer/testcontainers";
import { Account, getContract, GetContractReturnType, parseEther, parseUnits } from "viem";
import { OperatorNode } from "./operatorNode";
import { abi, bytecode } from "./out/BVS.sol/BVS.json";
import { abi as vaultAbi } from "@satlayer/contracts/out/SLAYVaultV2.sol/SLAYVaultV2.json";
import { abi as mockErc20Abi } from "@satlayer/contracts/out/MockERC20.sol/MockERC20.json";

let ethNodeStarted: StartedAnvilContainer;
let contracts: EVMContracts;

let owner: Account;
let operator1: Account;
let operator2: Account;
let staker: Account;

let vault1: `0x${string}`;
let vault2: `0x${string}`;

let wbtcContract: GetContractReturnType<typeof mockErc20Abi, SuperTestClient, `0x${string}`>;
const wbtcPriceId: `0x${string}` = "0xc9d8b075a5c69303365ae23633d4e085199bf5c520a3b90fed1322a0342ffc33";

let bvs: Awaited<ReturnType<typeof ethNodeStarted.deployContract>>;
let operator1Node: OperatorNode;
let operator2Node: OperatorNode;

async function initBVS(ethNodeStarted: StartedAnvilContainer): Promise<void> {
  bvs = await ethNodeStarted.deployContract({
    abi: abi,
    salt: "threshold-security",
    bytecode: bytecode.object as unknown as `0x${string}`,
    constructorArgs: [
      contracts.router.address,
      contracts.registry.address,
      contracts.oracle!.address,
      owner.address,
      {
        threshold: parseUnits("100000", 6), // 100k USD (in 6 decimals)
        exponent: 6,
      },
    ],
  });

  await ethNodeStarted.mineBlock(1);

  await bvs.contract.write.enableSlashing(
    [
      {
        destination: bvs.contractAddress,
        maxMbips: 100_000,
        resolutionWindow: 864000, // 10 days;
      },
    ],
    { account: owner.address },
  );

  await ethNodeStarted.mineBlock(1);

  // register operators to the BVS contract
  await bvs.contract.write.registerOperator([operator1.address], {
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

  // setup local evm node with SLAY contracts
  contracts = await EVMContracts.bootstrap(ethNodeStarted);
  await contracts.initOracle();

  owner = ethNodeStarted.getAccount() as unknown as Account;
  operator1 = ethNodeStarted.generateAccount("operator1") as unknown as Account;
  operator2 = ethNodeStarted.generateAccount("operator2") as unknown as Account;
  staker = ethNodeStarted.generateAccount("staker") as unknown as Account;

  // Fund all accounts with some gas
  await ethNodeStarted.setBalance(owner.address, parseEther("1"));
  await ethNodeStarted.setBalance(operator1.address, parseEther("1"));
  await ethNodeStarted.setBalance(operator2.address, parseEther("1"));
  await ethNodeStarted.setBalance(staker.address, parseEther("1"));

  await ethNodeStarted.mineBlock(1);

  // register operator
  await contracts.registry.write.registerAsOperator(["www.operator1.com", "operator1"], {
    account: operator1.address,
  });
  await contracts.registry.write.registerAsOperator(["www.operator2.com", "operator2"], {
    account: operator2.address,
  });
  await ethNodeStarted.mineBlock(1);

  // create WBTC asset
  const wbtcAsset = await contracts.initERC20({ name: "wbtc", decimals: 8, symbol: "WBTC" });
  wbtcContract = getContract({
    address: wbtcAsset.contractAddress,
    abi: mockErc20Abi,
    client: ethNodeStarted.getClient(),
  });

  // operators creates vaults
  vault1 = await contracts.initVault({ operator: operator1, underlyingAsset: wbtcAsset.contractAddress });
  vault2 = await contracts.initVault({ operator: operator2, underlyingAsset: wbtcAsset.contractAddress });

  // whitelist vaults
  await contracts.router.write.setVaultWhitelist([vault1, true]);
  await contracts.router.write.setVaultWhitelist([vault2, true]);

  // Initializes the BVS contract and registers it with the registry and operators.
  await initBVS(ethNodeStarted);
  await ethNodeStarted.mineBlock(1);

  {
    // expect bvs is init properly
    const threshold = await bvs.contract.read.threshold([]);
    expect(threshold).toStrictEqual([100000000000n, 6]);
  }

  operator1Node = new OperatorNode(
    "Operator1",
    ethNodeStarted.getClient(),
    ethNodeStarted,
    operator1,
    bvs.contractAddress,
  );
  operator2Node = new OperatorNode(
    "Operator2",
    ethNodeStarted.getClient(),
    ethNodeStarted,
    operator2,
    bvs.contractAddress,
  );

  // register service to operator
  await contracts.registry.write.registerServiceToOperator([bvs.contractAddress], {
    account: operator1.address,
  });
  await contracts.registry.write.registerServiceToOperator([bvs.contractAddress], {
    account: operator2.address,
  });

  // sets WBTC price to 100k USD
  const currentBlock = await ethNodeStarted.getClient().getBlock();
  await contracts.setOraclePrice({
    priceId: wbtcPriceId,
    price: parseUnits("100000", 8),
    conf: parseUnits("100", 8),
    expo: -8,
    timestamp: currentBlock.timestamp,
  });

  // mint 2 wbtc for staker
  await wbtcAsset.contract.write.mint([staker.address, parseUnits("2", 8)]);

  await ethNodeStarted.mineBlock(1);

  operator1Node.start();
  operator2Node.start();
}, 120_000);

afterAll(async () => {
  operator1Node.stop();
  operator2Node.stop();
  await ethNodeStarted.stop();
});

test("lifecycle", async () => {
  const client = ethNodeStarted.getClient();

  // set oracle mapping of asset to priceId
  await contracts.oracle.write.setPriceId([wbtcContract.address, wbtcPriceId]);
  await ethNodeStarted.mineBlock(1);

  // staker deposits to vault1 1 WBTC (worth 80k)
  await wbtcContract.write.approve([vault1, parseUnits("1", 8)], { account: staker });
  const vault1Contract = getContract({
    address: vault1,
    abi: vaultAbi,
    client: client,
  });
  await vault1Contract.write.deposit([parseUnits("1", 8), staker.address], { account: staker });

  // staker deposit to vault2 0.25 WBTC (worth 20k)
  await wbtcContract.write.approve([vault2, parseUnits("0.25", 8)], { account: staker });
  const vault2Contract = getContract({
    address: vault2,
    abi: vaultAbi,
    client: client,
  });
  await vault2Contract.write.deposit([parseUnits("0.25", 8), staker.address], { account: staker });

  await ethNodeStarted.mineBlock(1);

  // service request a response
  const requestId = await bvs.contract.simulate.request([5]);
  await bvs.contract.write.request([5]);
  await ethNodeStarted.mineBlock(1);

  // wait for all operators to respond
  await vi.waitFor(
    async () => {
      await ethNodeStarted.mineBlock(1);
      await bvs.contract.write.finalize([requestId.result, 25]);
    },
    {
      interval: 500,
      timeout: 10_000,
    },
  );
  await ethNodeStarted.mineBlock(1);

  {
    // expect tvl to be 100k
    const totalResponseTVL = await bvs.contract.read.responsesTVL([requestId.result, 25]);
    expect(totalResponseTVL).toStrictEqual(100_000_000_000n);
  }

  // get finalized response
  const finalized = await bvs.contract.read.finalizedRequests([requestId.result]);
  expect(finalized).toStrictEqual(true);
}, 10_000);
