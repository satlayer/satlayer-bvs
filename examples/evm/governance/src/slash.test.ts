import { beforeAll, expect, test, vi } from "vitest";
import { AnvilContainer, ChainName, EVMContracts, StartedAnvilContainer } from "@satlayer/testcontainers";
import { Account, encodeFunctionData, getContract, getEventSelector, pad, padHex, parseEther } from "viem";
import BVS from "./contracts/out/BVS.sol/BVS.json";
import { Committee } from "./committee";
import { abi as slayRouterAbi } from "@satlayer/contracts/SLAYRouterV2.sol/SLAYRouterV2.json";

let ethNodeStarted: StartedAnvilContainer;
let contracts: EVMContracts;

let committeeMembers: Account[];
let operator: Account;
let retailStaker: Account;
let bvsContract: Awaited<ReturnType<typeof ethNodeStarted.deployContract>>;
let committee: Committee;
let token: Awaited<ReturnType<typeof contracts.initERC20>>;
let vault: Awaited<ReturnType<typeof contracts.getVaultContractInstance>>;

// Initializes the BVS contract and registers it with the registry and operators.
async function initBVS(ethNodeStarted: StartedAnvilContainer) {
  // deploy BVS contract
  const committeeAddress = committeeMembers.map((v) => v.address);

  bvsContract = await ethNodeStarted.deployContract({
    abi: BVS.abi,
    bytecode: BVS.bytecode.object as unknown as `0x${string}`,
    salt: "bvs",
    constructorArgs: [committeeAddress, 4, contracts.registry.address, contracts.router.address],
  });

  await ethNodeStarted.mineBlock(1);

  await contracts.registry.write.registerAsOperator(["www.operator.com", "operator"], { account: operator.address });

  await ethNodeStarted.mineBlock(1);

  committee = new Committee(
    ethNodeStarted,
    getContract({
      address: bvsContract.contractAddress,
      abi: BVS.abi,
      client: ethNodeStarted.getClient(),
    }),
    contracts,
    committeeMembers,
  );

  await committee.registerOperator(operator.address);
  await committee.enableSlashing();
  await contracts.registry.write.registerServiceToOperator([bvsContract.contractAddress], {
    account: operator.address,
  });

  await ethNodeStarted.mineBlock(1);

  console.log(await contracts.registry.read.getRelationshipStatus([bvsContract.contractAddress, operator.address]));

  // init vaults for operator
  token = await contracts.initERC20({
    name: "Wrapped XYZ Token",
    symbol: "WXYZ",
    decimals: 16,
  });

  await ethNodeStarted.mineBlock(1);

  const vaultAddress = await contracts.initVault({
    operator: operator as unknown as Account,
    underlyingAsset: token.contract.address,
  });

  await contracts.router.write.setVaultWhitelist([vaultAddress, true], {
    account: ethNodeStarted.getAccount().address,
  });

  await ethNodeStarted.mineBlock(1);

  console.log("Vault whitelisted:", await contracts.router.read.isVaultWhitelisted([vaultAddress]));

  vault = await contracts.getVaultContractInstance(vaultAddress);

  await ethNodeStarted.mineBlock(1);

  await token.contract.write.mint([retailStaker.address, 100_000], { account: ethNodeStarted.getAccount().address });

  await ethNodeStarted.mineBlock(1);

  await token.contract.write.approve([vaultAddress, 100_000], { account: retailStaker.address });

  await ethNodeStarted.mineBlock(1);

  await vault.write.deposit([50_000, retailStaker.address], { account: retailStaker.address, gas: 300_000n });

  await ethNodeStarted.mineBlock(1);
}

beforeAll(async () => {
  // run local forked nodes
  let ethNode = new AnvilContainer({
    forkedChainName: ChainName.EthereumMainnet,
  });

  ethNodeStarted = await ethNode.start();

  contracts = await EVMContracts.bootstrap(ethNodeStarted);

  // generate accounts;
  committeeMembers = [];
  committeeMembers[0] = ethNodeStarted.generateAccount("member1") as unknown as Account;
  committeeMembers[1] = ethNodeStarted.generateAccount("member2") as unknown as Account;
  committeeMembers[2] = ethNodeStarted.generateAccount("member3") as unknown as Account;
  committeeMembers[3] = ethNodeStarted.generateAccount("member4") as unknown as Account;
  committeeMembers[4] = ethNodeStarted.generateAccount("member5") as unknown as Account;
  operator = ethNodeStarted.generateAccount("operator") as unknown as Account;
  retailStaker = ethNodeStarted.generateAccount("retailStaker") as unknown as Account;

  // give gas
  await ethNodeStarted.setBalance(operator.address, parseEther("1"));
  await ethNodeStarted.setBalance(retailStaker.address, parseEther("1"));

  for (const member of committeeMembers) {
    await ethNodeStarted.setBalance(member.address, parseEther("1"));
  }

  await ethNodeStarted.mineBlock(1);

  await initBVS(ethNodeStarted);
  await ethNodeStarted.mineBlock(1);
}, 120_000);

test("Reach a quorum to slash an operator.", async () => {
  // reaching consensus to request slash for the operator
  const calldata = encodeFunctionData({
    abi: slayRouterAbi,
    functionName: "requestSlashing",
    args: [
      {
        operator: operator.address,
        mbips: 500_000n, // 50%
        reason: "Committee deemed operator is malicious",
        timestamp: await ethNodeStarted.getClient().getBlockNumber(), // 1000 blocks in the future
      },
    ],
  });

  let slashId;
  committee
    .propose(contracts.router.address, 0n, calldata)
    .then(async (proposalId) => {
      await committee.allVoteYes(proposalId);
      const receipt = await committee.executeProposal(proposalId);
      const eventSelector = getEventSelector(
        "SlashingRequested(address,address,bytes32,(address,uint32,uint32),string)",
      );
      const slashingLog = receipt.logs.find((log) => log.topics[0] === eventSelector);
      if (!slashingLog) {
        throw new Error(`No SlashingRequested event found in transaction: ${proposalId}`);
      }
      console.log(slashingLog);
      slashId = BigInt(slashingLog.topics[2]!);
    })
    .catch((error) => {
      console.error(`Failed to slash operator: ${error.message}`);
    });

  console.log(`Slash ID: ${slashId}`);

  //----------------------------------
  // Reaching consensus to lock the slash collateral after request slashing
  // -----------------------------------
  // const calldata2 = encodeFunctionData({
  //     abi: slayRouterAbi,
  //     functionName: "lockSlashing",
  //     args: [operator.address]
  // });

  const balanceAfterSlash = await token.contract.read.balanceOf([vault.address]);
  console.log(`Balance after slash: ${balanceAfterSlash}`);
}, 120_000);
