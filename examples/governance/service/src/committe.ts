import { afterAll, beforeAll, expect, test, vi } from "vitest";
import { CosmWasmContainer, StartedCosmWasmContainer, SatLayerContracts } from "@satlayer/testcontainers";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { readFile } from "node:fs/promises";
import { stringToPath } from "@cosmjs/crypto";
import { ExecuteMsg, InstantiateMsg, Voter } from "@examples/governance-contract/governance-contract";

export type CommitteeMember = Voter & { wallet: DirectSecp256k1HdWallet };

export const generateWallet = async (count: number) => {
  const wallets: DirectSecp256k1HdWallet[] = [];
  for (let i = 0; i < count; i++) {
    const wallet = await DirectSecp256k1HdWallet.generate(12, {
      prefix: "wasm",
      hdPaths: [stringToPath(`m/${i}`)],
    });
    wallets.push(wallet);
  }
  return wallets;
};

export const generateCommittee = async (count: number): Promise<CommitteeMember[]> => {
  const committee: CommitteeMember[] = [];
  let wallets = await generateWallet(count);
  for (let i = 0; i < count; i++) {
    const [account] = await wallets[i].getAccounts();
    committee.push({
      addr: account.address,
      weight: 1,
      wallet: wallets[i],
    });
  }
  return committee;
};
