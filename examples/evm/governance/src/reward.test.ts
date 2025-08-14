import { beforeAll, expect, test } from "vitest";
import { AnvilContainer, ChainName, EVMContracts, StartedAnvilContainer } from "@satlayer/testcontainers";
import { Account, encodeFunctionData, getContract, parseEther } from "viem";
import BVS from "./contracts/out/BVS.sol/BVS.json";
import { Committee } from "./committee";
import { DistributionRewards, Earner, findProjectRoot, offChainRewardTrigger } from "./reward";
import { execa } from "execa";

let ethNodeStarted: StartedAnvilContainer;
let contracts: EVMContracts;

let committeeMembers: Account[];
let operator: Account;
let retailStakers: Account[];
let bvsContract: Awaited<ReturnType<typeof ethNodeStarted.deployContract>>;
let committee: Committee;
let token: Awaited<ReturnType<typeof contracts.initERC20>>;
let vault: Awaited<ReturnType<typeof contracts.getVaultContractInstance>>;
let guardrailer: Account;
let earners: Earner[] = [];

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

  vault = await contracts.getVaultContractInstance(vaultAddress);

  await ethNodeStarted.mineBlock(1);

  for (const retailStaker of retailStakers) {
    await token.contract.write.mint([retailStaker.address, 100_000], { account: ethNodeStarted.getAccount().address });

    await ethNodeStarted.mineBlock(1);

    await token.contract.write.approve([vaultAddress, 100_000], { account: retailStaker.address });

    await ethNodeStarted.mineBlock(1);

    await vault.write.deposit([50_000, retailStaker.address], { account: retailStaker.address, gas: 300_000n });

    await ethNodeStarted.mineBlock(1);

    earners.push({
      account: retailStaker,
      staked: 50_000n,
    });
  }

  await contracts.router.write.setGuardrail([guardrailer.address], { account: ethNodeStarted.getAccount().address });

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

  retailStakers = [];
  for (let i = 0; i < 15; i++) {
    retailStakers[i] = ethNodeStarted.generateAccount(`retailStaker${i}`) as unknown as Account;
    await ethNodeStarted.setBalance(retailStakers[i].address, parseEther("1"));
  }

  guardrailer = ethNodeStarted.generateAccount("guardrailer") as unknown as Account;

  // give gas
  await ethNodeStarted.setBalance(operator.address, parseEther("1"));
  await ethNodeStarted.setBalance(guardrailer.address, parseEther("1"));

  for (const member of committeeMembers) {
    await ethNodeStarted.setBalance(member.address, parseEther("1"));
  }

  await ethNodeStarted.mineBlock(1);

  await initBVS(ethNodeStarted);
  await ethNodeStarted.mineBlock(1);
}, 120_000);

async function multiSigRewardDistrbution(merkleRoot: string, distributionData: DistributionRewards) {
  // console.log("MultiSig reward distribution started...", merkleRoot);
  await ethNodeStarted.setBalance(bvsContract.contractAddress, parseEther("1"));

  // For the simplicity of this example, the underlying asset to be distributed rewards
  // let's imagine is funded by the multi-sig member

  await token.contract.write.mint([bvsContract.contractAddress, 10_000_000], {
    account: ethNodeStarted.getAccount().address,
  });
  await ethNodeStarted.mineBlock(2);
  await token.contract.write.approve([contracts.rewards.address, 10_000_000], { account: bvsContract.contractAddress });
  await ethNodeStarted.mineBlock(2);
  const calldata_toDistribute = encodeFunctionData({
    abi: contracts.rewards.abi,
    functionName: "distributeRewards",
    args: [token.contractAddress, BigInt(distributionData.totalReward), `0x${merkleRoot}`],
  });

  const distribution_proposalId = await committee.propose(contracts.rewards.address, 0n, calldata_toDistribute);
  await committee.allVoteYes(distribution_proposalId);
  await committee.executeProposal(distribution_proposalId);
}

test("Reward distribution lifecycle", async () => {
  const { merkleRoot, distributionData } = await offChainRewardTrigger(
    token.contract.address,
    bvsContract.contractAddress,
    earners,
    multiSigRewardDistrbution,
  );

  const distDir = `/dist/eth-mainnet/${bvsContract.contractAddress}/${token.contractAddress}/distribution.json`;
  let rootDir = findProjectRoot();
  let distributionFilePath = rootDir + distDir;

  const { stdout } = await execa(
    "satlayer",
    [
      "rewards",
      "proof",
      distributionData.earners[0].earner,
      distributionData.earners[0].reward,
      "-f",
      distributionFilePath,
    ],
    { preferLocal: true },
  );
  let proofRes = JSON.parse(stdout.trim()).claim_rewards_proof;

  let cliamableProof = {
    provider: bvsContract.contractAddress,
    token: token.contractAddress,
    amount: distributionData.earners[0].reward,
    merkleRoot: `0x${proofRes.root}`,
    proof: proofRes.proof.map((v: string) => `0x${v}`),
    leafIndex: proofRes.leaf_index,
    totalLeaves: proofRes.total_leaves_count,
    recipient: distributionData.earners[0].earner,
  };

  await contracts.rewards.write.claimRewards([cliamableProof], { account: retailStakers[0].address, gas: 300_000n });
  await ethNodeStarted.mineBlock(1);

  const balance = await token.contract.read.balanceOf([retailStakers[0].address]);

  expect(balance).toBe(50_416n); // Total minted - Initial stake + claimed rewards
}, 120_000);
