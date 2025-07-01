import { execa } from "execa";
import { Api } from "./api";
import { AllAccountsResponse } from "@satlayer/cosmwasm-schema/vault-bank-tokenized";
import * as path from "node:path";
import * as fs from "node:fs";

/**
 * Rewards are calculated and scheduled off-chain.
 * Default is 10% APY, calculated monthly.
 * For this example, we will emulate that
 * Off-chain service will calculate the rewards
 * And each month, the multi-sig member will review
 * and approve the rewards and then inject into the BVS
 * For the simplicity of this example, calculated reward amount each
 * the multi sig member will pay out of pocket and fund the BVS contract
 * Which the bvs contract will allow the stakers to claim their rewards
 **/
const APY_PERCENT = 10; // 10% APY, representing as full digits for ease of working with bigint.
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

/**
 * This function is called by the off-chain service to trigger the reward distribution.
 * It calculates the rewards based on the staked amounts and creates a Merkle tree.
 * The Merkle root and distribution data are then passed to the callback function.
 * For this example, the callback function will be the multi-sig member's consensus to approve the rewards.
 *
 * @param api - The API instance to interact with the blockchain
 * @param callback - The callback function to handle the Merkle root and distribution data
 * @returns An object containing the Merkle root and distribution data
 */
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

/**
 * This function simply calculate how much reward a particular staker should get
 * given their staked amount (tvl) and the APY.
 * Meant be called once every month to calculate the rewards in this example.
 *
 * @param tvl - The total value locked (TVL) in the vault for the staker
 * @param apy - The annual percentage yield (APY) in percent (default is 10%)
 * @param interestComputeInterval - The number of intervals in a year (default is 12 for monthly)
 * @returns The calculated reward amount in the smallest unit of the token (e.g., uStake)
 */
function calculateReward(
  tvl: bigint,
  apy: number = APY_PERCENT,
  interestComputeInterval: number = INTEREST_COMPUTE_INTERVAL_PER_ANNUAL,
): bigint {
  const numerator = tvl * BigInt(apy);
  const denominator = BigInt(100) * BigInt(interestComputeInterval);

  return numerator / denominator;
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

/**
 * This function dumps the reward distribution data into a JSON file.
 * It retrieves all stakers' addresses from SatLayer protocol, calculates their staked amounts,
 * and computes their rewards based on the APY.
 * The resulting distribution data is saved in a file named distribution.json
 * in the dist/bbn-test-5/{api.Service}/ustake directory.
 * For the simplicity of this example, the operator is not rewarded if the operator hasn't staked any tokens.
 *
 *
 * @param api - The API instance to interact with the blockchain
 * @returns The distribution rewards data
 */
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
