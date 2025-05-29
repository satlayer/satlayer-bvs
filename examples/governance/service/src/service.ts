import { execa } from "execa";
import { Api } from "./api";
import { AllAccountsResponse } from "@satlayer/cosmwasm-schema/vault-bank-tokenized";
import * as path from "node:path";
import * as fs from "node:fs";

const APY = 0.1;
const INTEREST_COMPUTE_INTERVAL_PER_ANNUAL = 365;

export interface DistributionRewards {
  // The token to be distributed
  token: string;
  // The list of earners with their rewards
  earners: Array<{
    earner: string; // address of the earner
    reward: string; // amount of reward in string format
  }>;
}

export async function offChainRewardTrigger(
  api: Api,
  callback: (merkleRoot: String, distributionData: DistributionRewards) => void,
) {
  const distributionFileData = await dumpRewardDistribution(api);
  const merkleRoot = await createMerkleTree(
    path.resolve(process.cwd(), `dist/bbn-test-5/${api.Service}/ustake/distribution.json`),
  );

  callback(merkleRoot, distributionFileData);
}

function calculateReward(
  tvl: bigint,
  apy: number = APY,
  interestComputeInterval: number = INTEREST_COMPUTE_INTERVAL_PER_ANNUAL,
): bigint {
  // Calculate the reward based on the TVL and the APY
  // REWARD_PER_PERIOD is the daily reward rate
  // tvl is in uStake, so we need to convert it to a number for calculation
  return tvl * BigInt(Math.floor(apy / interestComputeInterval)); // Assuming 1e6 is the scaling factor for uStake
}

/**
 * Creates a Merkle tree from the distribution.json file using the satlayer CLI tool.
 *
 * It uses the `satlayer rewards create` command to generate the Merkle root from the distribution.json file.
 *
 * @param inputFile The path to the distribution.json file
 * @returns The Merkle root hash as a string
 */
async function createMerkleTree(inputFile: string) {
  let binPath = require.resolve("@satlayer/cli/node_modules/@modules/cosmwasm-cli/dist/cosmwasm-cli");
  const { stdout } = await execa(binPath, ["rewards", "create", "-f", inputFile], { preferLocal: true });

  // Parse the Merkle root line
  const match = stdout.match(/Merkle root:\s*([0-9a-fA-F]{64})/);
  if (!match) {
    throw new Error("Failed to parse Merkle root from output");
  }

  return match[1];
}

async function dumpRewardDistribution(api: Api): Promise<DistributionRewards> {
  let now = Date.now();

  // get all vault bank stakers address
  let allAccountsRes: AllAccountsResponse = await api.queryVaultAllAccounts();

  // get each staker's balance and calculate total staked amount
  let stakerTvlMap = new Map<string, bigint>();
  let totalStaked = BigInt(0);
  for (let account of allAccountsRes.accounts) {
    // get each staker receipt token balance ( = amount staked )
    let stakerShare = await api.queryStakedAmount({ address: account });
    stakerTvlMap.set(account, stakerShare);
    totalStaked += stakerShare;
  }

  if (totalStaked === BigInt(0)) {
    throw new Error("No stakers found or total staked amount is zero");
  }

  // calculate rewards for each staker proportional to their stake/balance
  let stakerRewardsMap = new Map<string, bigint>();
  for (let account of allAccountsRes.accounts) {
    let staker_tvl = stakerTvlMap.get(account) || BigInt(0);
    let stakerRewardsAmount = calculateReward(staker_tvl); // convert to uStake
    stakerRewardsMap.set(account, stakerRewardsAmount);
  }

  // convert to distribution.json file format
  let newDistributionFileData: DistributionRewards = {
    token: "ustake",
    earners: Array.from(stakerRewardsMap.entries()).map(([earner, reward]) => ({
      earner: earner,
      reward: reward.toString(),
    })),
  };

  // write the new distribution.json into the dist folder
  const distDir = path.resolve(process.cwd(), `dist/bbn-test-5/${api.Service}/ustake`);
  fs.mkdirSync(distDir, { recursive: true });

  const filePath = path.join(distDir, "distribution.json");
  fs.writeFileSync(filePath, JSON.stringify(newDistributionFileData, null, 2), "utf8");

  return newDistributionFileData;
}
