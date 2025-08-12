import { AnvilContainer, EVMContracts, StartedAnvilContainer, SuperTestClient } from "@satlayer/testcontainers";
import { Account, encodeFunctionData, GetContractReturnType, getEventSelector, TransactionReceipt } from "viem";
import { abi } from "./contracts/out/BVS.sol/BVS.json";
import { abi as slayRegistryAbi } from "@satlayer/contracts/SLAYRegistryV2.sol/SLAYRegistryV2.json";

export class Committee {
  private readonly ethNodeStarted: StartedAnvilContainer;
  private readonly bvsContract: GetContractReturnType<typeof abi, any, `0x${string}`>;
  private readonly comittee: Account[];
  private readonly slayContracts: EVMContracts;

  constructor(
    ethNodeStarted: StartedAnvilContainer,
    bvsContract: GetContractReturnType<typeof abi, any, `0x${string}`>,
    slayContracts: EVMContracts,
    comittee: Account[],
  ) {
    this.ethNodeStarted = ethNodeStarted;
    this.bvsContract = bvsContract;
    this.comittee = comittee;
    this.slayContracts = slayContracts;
  }

  async propose(destination: string, value: BigInt, calldata: `0x${string}`): Promise<BigInt> {
    const txHash = await this.bvsContract.write.submitProposal([destination, value, calldata], {
      account: this.comittee[0].address,
    });

    await this.ethNodeStarted.mineBlock(2);

    const receipt = await this.ethNodeStarted.getClient().waitForTransactionReceipt({ hash: txHash });
    if (!receipt.status) {
      throw new Error(`Transaction failed: ${txHash}`);
    }
    const eventSelector = getEventSelector("Submission(uint256)");
    const submissionLog = receipt.logs.find((log) => log.topics[0] === eventSelector);

    if (!submissionLog) {
      throw new Error(`No submission event found in transaction: ${txHash}`);
    }

    return BigInt(submissionLog.topics[1]!);
  }

  async allVoteYes(proposal_id: BigInt): Promise<void> {
    for (let i = 1; i < this.comittee.length; i++) {
      const txHash = await this.bvsContract.write.confirmProposal([proposal_id], { account: this.comittee[i].address });
      await this.ethNodeStarted.mineBlock(2);
      const receipt = await this.ethNodeStarted.getClient().waitForTransactionReceipt({ hash: txHash });
      if (receipt.status != "success") {
        throw new Error(`Proposal Voting Transaction failed: ${txHash}`);
      }
    }
  }

  async executeProposal(proposal_id: BigInt): Promise<TransactionReceipt> {
    const txHash = await this.bvsContract.write.executeProposal([proposal_id], { account: this.comittee[0].address });
    await this.ethNodeStarted.mineBlock(2);
    const receipt = await this.ethNodeStarted.getClient().waitForTransactionReceipt({ hash: txHash });
    console.log(receipt);
    if (receipt.status != "success") {
      throw new Error(`Proposal Execution failed: ${txHash}`);
    }
    return receipt;
  }

  async enableSlashing(): Promise<void> {
    const slashParameter = {
      destination: this.bvsContract.address,
      maxMbips: 10_000_000n,
      resolutionWindow: 80_000,
    };
    const calldata = encodeFunctionData({
      abi: slayRegistryAbi,
      functionName: "enableSlashing",
      args: [slashParameter],
    });
    this.propose(this.slayContracts.registry.address, 0n, calldata)
      .then(async (proposalId) => {
        await this.allVoteYes(proposalId);
        await this.executeProposal(proposalId);
      })
      .catch((error) => {
        console.error(`Failed to enable slashing: ${error.message}`);
      });
  }

  async registerOperator(operator: String): Promise<void> {
    const calldata = encodeFunctionData({
      abi: slayRegistryAbi,
      functionName: "registerOperatorToService",
      args: [operator],
    });
    const destination = this.slayContracts.registry.address;
    const txHash = await this.bvsContract.write.submitProposal([destination, 0, calldata], {
      account: this.comittee[0].address,
    });

    console.log(`Transaction hash: ${txHash}`);
    await this.ethNodeStarted.mineBlock(2);

    const receipt = await this.ethNodeStarted.getClient().waitForTransactionReceipt({ hash: txHash });
    if (!receipt.status) {
      throw new Error(`Transaction failed: ${txHash}`);
    }
    const eventSelector = getEventSelector("Submission(uint256)");
    const submissionLog = receipt.logs.find((log) => log.topics[0] === eventSelector);

    if (!submissionLog) {
      throw new Error(`No submission event found in transaction: ${txHash}`);
    }

    const proposalId = BigInt(submissionLog.topics[1]!);
    await this.allVoteYes(proposalId);
    await this.executeProposal(proposalId);
  }
}
