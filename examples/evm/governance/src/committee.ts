import { EVMContracts, StartedAnvilContainer } from "@satlayer/testcontainers";
import {
  Account,
  BaseError,
  encodeFunctionData,
  getContractError,
  GetContractReturnType,
  getEventSelector,
  TransactionReceipt,
} from "viem";
import { abi } from "./contracts/out/BVS.sol/BVS.json";
import { abi as slayRegistryAbi } from "@satlayer/contracts/out/SLAYRegistryV2.sol/SLAYRegistryV2.json";

export class Committee {
  private readonly ethNodeStarted: StartedAnvilContainer;
  private readonly bvsContract: GetContractReturnType<typeof abi, any, `0x${string}`>;
  private readonly committee: Account[];
  private readonly slayContracts: EVMContracts;

  constructor(
    ethNodeStarted: StartedAnvilContainer,
    bvsContract: GetContractReturnType<typeof abi, any, `0x${string}`>,
    slayContracts: EVMContracts,
    comittee: Account[],
  ) {
    this.ethNodeStarted = ethNodeStarted;
    this.bvsContract = bvsContract;
    this.committee = comittee;
    this.slayContracts = slayContracts;
  }

  async propose(destination: string, value: BigInt, calldata: `0x${string}`): Promise<BigInt> {
    const txHash = await this.bvsContract.write.submitProposal([destination, value, calldata], {
      account: this.committee[0].address,
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
    for (let i = 1; i < this.committee.length; i++) {
      const txHash = await this.bvsContract.write.confirmProposal([proposal_id], {
        account: this.committee[i].address,
      });
      await this.ethNodeStarted.mineBlock(2);
      const receipt = await this.ethNodeStarted.getClient().waitForTransactionReceipt({ hash: txHash });
      if (receipt.status != "success") {
        throw new Error(`Proposal Voting Transaction failed: ${txHash}`);
      }
    }
  }

  async executeProposal(proposal_id: BigInt): Promise<TransactionReceipt> {
    const txHash = await this.bvsContract.write.executeProposal([proposal_id], { account: this.committee[0].address });
    await this.ethNodeStarted.mineBlock(2);
    const receipt = await this.ethNodeStarted.getClient().waitForTransactionReceipt({ hash: txHash });
    if (receipt.status != "success") {
      await this.ethNodeStarted
        .getClient()
        .simulateContract({
          address: this.bvsContract.address,
          abi: this.bvsContract.abi,
          functionName: "executeProposal",
          args: [proposal_id],
          account: this.committee[0].address,
          blockNumber: receipt.blockNumber,
        })
        .catch((error) => {
          throw getContractError(error as BaseError, {
            abi: this.bvsContract.abi,
            address: this.bvsContract.address,
            sender: this.committee[0].address,
            functionName: "executeProposal",
            args: [proposal_id],
          });
        });
    }
    return receipt;
  }

  async enableSlashing(): Promise<void> {
    const slashParameter = {
      destination: this.bvsContract.address,
      maxMbips: 10_000_000n,
      resolutionWindow: 32,
    };
    const calldata = encodeFunctionData({
      abi: slayRegistryAbi,
      functionName: "enableSlashing",
      args: [slashParameter],
    });
    const proposalId = await this.propose(this.slayContracts.registry.address, 0n, calldata);
    await this.allVoteYes(proposalId);
    await this.executeProposal(proposalId);
  }

  async registerOperator(operator: String): Promise<void> {
    const calldata = encodeFunctionData({
      abi: slayRegistryAbi,
      functionName: "registerOperatorToService",
      args: [operator],
    });
    const destination = this.slayContracts.registry.address;
    const txHash = await this.bvsContract.write.submitProposal([destination, 0, calldata], {
      account: this.committee[0].address,
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

    const proposalId = BigInt(submissionLog.topics[1]!);
    await this.allVoteYes(proposalId);
    await this.executeProposal(proposalId);
  }
}
