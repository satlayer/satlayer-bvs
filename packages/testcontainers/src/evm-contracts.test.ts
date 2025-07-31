import { afterEach, beforeEach, expect, test } from "vitest";

import { AnvilContainer, ChainName, StartedAnvilContainer } from "./anvil-container";
import { EVMContracts } from "./evm-contracts";

let started: StartedAnvilContainer;
let evmContracts: EVMContracts;

beforeEach(async () => {
  const ethNode = new AnvilContainer({
    forkedChainName: ChainName.EthereumMainnet,
  });
  started = await ethNode.start();
  evmContracts = await EVMContracts.bootstrap(started);
}, 30_000);

afterEach(async () => {
  await started.stop();
});

test("should bootstrap contracts", async () => {
  // check if registry is deployed
  const randomAddress = started.getRandomAddress();
  const isOperator = await evmContracts.registry.read.isOperator([randomAddress]);
  expect(isOperator).toBeFalsy();

  // check if router is deployed
  const isVaultWhitelisted = await evmContracts.router.read.isVaultWhitelisted([randomAddress]);
  expect(isVaultWhitelisted).toBeFalsy();

  // check if rewards is deployed
  const distributionRoot = await evmContracts.rewards.read.getDistributionRoots([randomAddress, randomAddress]);
  expect(distributionRoot).toStrictEqual({
    currentRoot: "0x0000000000000000000000000000000000000000000000000000000000000000",
    prevRoot: "0x0000000000000000000000000000000000000000000000000000000000000000",
  });
}, 30_000);

test("should init vault", async () => {
  const operator = started.generateAccount("operator");
  await started.setBalance(operator.address, BigInt(1e18)); // 1 ETH

  // register operator
  await evmContracts.registry.write.registerAsOperator(["www.operator.com", "operator"], {
    account: operator,
  });

  // create underlying erc20 token
  const wbtc = await evmContracts.initERC20({
    name: "Wrapped Bitcoin",
    symbol: "WBTC",
    decimals: 8,
  });

  // init vault
  const vaultAddress = await evmContracts.initVault({
    operator: operator,
    underlyingAsset: wbtc.contractAddress,
  });
  expect(vaultAddress).toBeDefined();

  // get vault contract instance
  const vaultContract = await evmContracts.getVaultContractInstance(vaultAddress);

  // whitelist vault in router
  await evmContracts.router.write.setVaultWhitelist([vaultAddress, true]);
  await started.mineBlock(1); // mine a block to ensure the transaction is processed

  // expect vault is whitelisted
  const isVaultWhitelisted = await evmContracts.router.read.isVaultWhitelisted([vaultAddress]);
  expect(isVaultWhitelisted).toBeTruthy();

  // mint some WBTC to a staker
  const staker = started.generateAccount("staker");
  await started.setBalance(staker.address, BigInt(1e18)); // fund 1 ETH to staker for gas
  await wbtc.contract.write.mint([staker.address, 1e8]);

  // staker stakes WBTC in the vault
  await wbtc.contract.write.approve([vaultAddress, 1e8], { account: staker });
  await vaultContract.write.deposit([1e8, staker.address], { account: staker });
  await started.mineBlock(1); // mine a block to ensure the transaction is processed

  // assert staker balance in vault
  const stakerBalance = await vaultContract.read.balanceOf([staker.address]);
  expect(stakerBalance).toStrictEqual(BigInt(1e8));
});
