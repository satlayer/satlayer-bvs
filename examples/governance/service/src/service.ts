import { execa } from "execa";
import { Api } from "./api";
import { AllAccountsResponse } from "@satlayer/cosmwasm-schema/vault-bank-tokenized";
import * as path from "node:path";
import * as fs from "node:fs";

const APY_PERCENT = 10; // 10% APY
const INTEREST_COMPUTE_INTERVAL_PER_ANNUAL = 12;

export interface DistributionRewards {
  // The token to be distributed
  token: string;
  // The list of earners with their rewards
  earners: Array<{
    earner: string; // address of the earner
    reward: string; // amount of reward in string format
  }>;

  totalReward: string; // optional total reward amount
}

export async function offChainRewardTrigger(
  api: Api,
  callback: (merkleRoot: string, distributionData: DistributionRewards) => Promise<void>,
): Promise<{ merkleRoot: string; distributionData: DistributionRewards }> {
  const distributionFileData = await dumpRewardDistribution(api);
  const distDir = `/dist/bbn-test-5/${api.Service}/ustake/distribution.json`;
  let rootDir = findProjectRoot();
  let distributionFilePath = rootDir + distDir;
  const merkleRoot = await createMerkleTree(distributionFilePath);

  await callback(merkleRoot, distributionFileData);

  return { merkleRoot, distributionData: distributionFileData };
}

function calculateReward(
  tvl: bigint,
  apy: number = APY_PERCENT,
  interestComputeInterval: number = INTEREST_COMPUTE_INTERVAL_PER_ANNUAL,
): bigint {
  // Calculate the reward based on the TVL and the APY
  // REWARD_PER_PERIOD is the daily reward rate
  // tvl is in uStake, so we need to convert it to a number for calculation
  const numerator = tvl * BigInt(apy);
  const denominator = BigInt(100) * BigInt(interestComputeInterval);

  return numerator / denominator; // Convert to uStake with 6 decimals
}

/**
 * Creates a Merkle tree from the distribution.json file using the satlayer CLI tool.
 *
 * It uses the `satlayer rewards create` command to generate the Merkle root from the distribution.json file.
 *
 * @param distFilePath The path to the distribution.json file
 * @returns The Merkle root hash as a string
 */
async function createMerkleTree(distFilePath: string): Promise<string> {
  const { stdout } = await execa("satlayer", ["rewards", "create", "-f", distFilePath], { preferLocal: true });
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
  let cumulativeReward = BigInt(0);
  for (let account of allAccountsRes.accounts) {
    let staker_tvl = stakerTvlMap.get(account);
    let stakerRewardsAmount = calculateReward(staker_tvl as bigint); // convert to uStake
    stakerRewardsMap.set(account, stakerRewardsAmount);
    cumulativeReward += stakerRewardsAmount;
  }

  // convert to distribution.json file format
  let newDistributionFileData: DistributionRewards = {
    token: "ustake",
    earners: Array.from(stakerRewardsMap.entries()).map(([earner, reward]) => ({
      earner: earner,
      reward: reward.toString(),
    })),
    totalReward: cumulativeReward.toString(),
  };

  // write the new distribution.json into the dist folder
  const distDir = path.resolve(process.cwd(), `dist/bbn-test-5/${api.Service}/ustake`);
  fs.mkdirSync(distDir, { recursive: true });

  const filePath = path.join(distDir, "distribution.json");
  fs.writeFileSync(filePath, JSON.stringify(newDistributionFileData, null, 2), "utf8");

  return newDistributionFileData;
}

export function findProjectRoot(startDir = __dirname) {
  let dir = startDir;
  // walk up until filesystem root
  while (dir !== path.parse(dir).root) {
    if (fs.existsSync(path.join(dir, "package.json"))) {
      return dir;
    }
    dir = path.dirname(dir);
  }
  throw new Error("Could not locate project root (no package.json found).");
}
