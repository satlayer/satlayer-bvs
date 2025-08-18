import { assert, beforeAll, test } from "vitest";
import { AnvilContainer, ChainName, EVMContracts, StartedAnvilContainer } from "@satlayer/testcontainers";
import { Account, encodeFunctionData, getContract, parseEther, parseEventLogs } from "viem";
import BVS from "./contracts/out/BVS.sol/BVS.json";
import { Committee } from "./committee";
import { abi as slayRouterAbi } from "@satlayer/contracts/out/SLAYRouterV2.sol/SLAYRouterV2.json";

let ethNodeStarted: StartedAnvilContainer;
let contracts: EVMContracts;

let committeeMembers: Account[];
let operator: Account;
let retailStaker: Account;
let bvsContract: Awaited<ReturnType<typeof ethNodeStarted.deployContract>>;
let committee: Committee;
let token: Awaited<ReturnType<typeof contracts.initERC20>>;
let vault: Awaited<ReturnType<typeof contracts.getVaultContractInstance>>;
let guardrailer: Account;

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

  await token.contract.write.mint([retailStaker.address, 100_000], { account: ethNodeStarted.getAccount().address });

  await ethNodeStarted.mineBlock(1);

  await token.contract.write.approve([vaultAddress, 100_000], { account: retailStaker.address });

  await ethNodeStarted.mineBlock(1);

  await vault.write.deposit([50_000, retailStaker.address], { account: retailStaker.address, gas: 300_000n });

  await ethNodeStarted.mineBlock(1);

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
  retailStaker = ethNodeStarted.generateAccount("retailStaker") as unknown as Account;
  guardrailer = ethNodeStarted.generateAccount("guardrailer") as unknown as Account;

  // give gas
  await ethNodeStarted.setBalance(operator.address, parseEther("1"));
  await ethNodeStarted.setBalance(retailStaker.address, parseEther("1"));
  await ethNodeStarted.setBalance(guardrailer.address, parseEther("1"));

  for (const member of committeeMembers) {
    await ethNodeStarted.setBalance(member.address, parseEther("1"));
  }

  await ethNodeStarted.mineBlock(1);

  await initBVS(ethNodeStarted);
  await ethNodeStarted.mineBlock(1);
}, 120_000);

test("Quorum to slash an operator.", async () => {
  // --------------------------------------------------------
  // Reaching consensus to request slash for the operator
  // --------------------------------------------------------
  const calldata = encodeFunctionData({
    abi: slayRouterAbi,
    functionName: "requestSlashing",
    args: [
      {
        operator: operator.address,
        mbips: 5_000_000n, // 50%
        reason: "Committee deemed operator is malicious",
        timestamp: (await ethNodeStarted.getClient().getBlock()).timestamp,
      },
    ],
  });

  let slashRequest_proposalId = await committee.propose(contracts.router.address, 0n, calldata);
  await committee.allVoteYes(slashRequest_proposalId);
  const slashRequest_receipt = await committee.executeProposal(slashRequest_proposalId);
  const parsedEvents = parseEventLogs({
    abi: slayRouterAbi,
    logs: slashRequest_receipt.logs,
    eventName: "SlashingRequested",
  });
  const slashId = parsedEvents[0].args.slashId;

  // Operator did not refute
  // Or weren't able to refute in time
  await ethNodeStarted.mineBlock(1000);

  //-----------------------------------------------------------------------
  // Reaching consensus to lock the slash collateral after request slashing
  // ----------------------------------------------------------------------
  const calldata2 = encodeFunctionData({
    abi: slayRouterAbi,
    functionName: "lockSlashing",
    args: [slashId],
  });
  const slashLock_proposalId = await committee.propose(contracts.router.address, 0n, calldata2);
  await committee.allVoteYes(slashLock_proposalId);
  await committee.executeProposal(slashLock_proposalId);

  //-----------------------------------------------------------------------
  // Reaching consensus to finalize slashing to the operator
  // ----------------------------------------------------------------------
  const calldata3 = encodeFunctionData({
    abi: slayRouterAbi,
    functionName: "finalizeSlashing",
    args: [slashId],
  });
  const slashFinalize_proposalId = await committee.propose(contracts.router.address, 0n, calldata3);
  await committee.allVoteYes(slashFinalize_proposalId);

  // Although in the example the guardrailer is just an EOA account,
  // in practice it should be a contract that implements the guardrail logic or
  // even independent committee separate from governance committee in this example.
  await contracts.router.write.guardrailApprove([slashId, true], { account: guardrailer.address });
  await ethNodeStarted.mineBlock(1);

  await committee.executeProposal(slashFinalize_proposalId);

  const balanceAfterSlash = await token.contract.read.balanceOf([vault.address]);
  const serviceBalanceAfterSlash = await token.contract.read.balanceOf([bvsContract.contractAddress]);
  assert(balanceAfterSlash === 25000n, "Vault balance should be half after slashing the operator");
  assert(
    serviceBalanceAfterSlash === 25000n,
    "BVS contract balance should be half of token the taken away from vault after slashing the operator",
  );
}, 120_000);
