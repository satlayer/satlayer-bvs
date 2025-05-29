import { afterAll, beforeAll, expect, test, vi } from "vitest";
import { CosmWasmContainer, StartedCosmWasmContainer, SatLayerContracts } from "@satlayer/testcontainers";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Coin, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { readFile } from "node:fs/promises";
import { stringToPath } from "@cosmjs/crypto";

import {
  Vote,
  ExecuteMsg as GovernanceExecuteMsg,
  InstantiateMsg,
  Voter,
} from "@examples/governance-contract/governance-contract";

import {
  ExecuteMsg as RouterExecuteMsg,
  QueryMsg as RouterQueryMsg,
  SlashingRequestResponse,
} from "@satlayer/cosmwasm-schema/vault-router";

import {
  ExecuteMsg as RegistryExecuteMsg,
  QueryMsg as RegistryQueryMsg,
  SlashingParametersResponse,
} from "@satlayer/cosmwasm-schema/registry";

import { ExecuteMsg as VaultBankExecuteMsg } from "@satlayer/cosmwasm-schema/vault-bank";

import { ExecuteMsg as GuardrailExecuteMsg } from "@satlayer/cosmwasm-schema/guardrail";
import { Api } from "./api";
import { offChainRewardTrigger } from "./service";

let started: StartedCosmWasmContainer;
let contracts: SatLayerContracts;
let bvs_wallet: DirectSecp256k1HdWallet;
let satlayer_wallet: DirectSecp256k1HdWallet;
let clientSigner: SigningCosmWasmClient;
let governanceContractAddress: string;
let committee: Voter[];
let vaultAddress: string;
let api: Api;

async function deployGovernanceContract(owner: string, committee: Voter[]) {
  const contractPath = require.resolve("@examples/governance-contract/dist/contract.wasm");
  const uploaded = await clientSigner.upload(owner, await readFile(contractPath), "auto");
  const initMsg: InstantiateMsg = {
    registry: contracts.registry.address,
    router: contracts.router.address,
    owner: owner,
    cw3_instantiate_msg: {
      voters: committee,
      threshold: {
        absolute_percentage: {
          percentage: "1",
        },
      },
      max_voting_period: {
        height: 100, // 24 hours in seconds
      }, // 24 hours
    },
  };
  return clientSigner.instantiate(owner, uploaded.codeId, initMsg, "governance", "auto");
}

async function setup_staking() {
  let [bvs_owner, _operator, staker] = await bvs_wallet.getAccounts();

  let msg: VaultBankExecuteMsg = {
    deposit_for: {
      recipient: staker.address,
      amount: "20000",
    },
  };

  let coin: Coin = {
    denom: "ustake",
    amount: "20000",
  };

  return clientSigner.execute(bvs_owner.address, vaultAddress, msg, "auto", undefined, [coin]);
}

beforeAll(async () => {
  // Set up CosmWasmContainer with SatLayerContracts bootstrapped
  started = await new CosmWasmContainer().start();
  contracts = await SatLayerContracts.bootstrap(started);

  // A wallet with 3 accounts, owner of contract, operator, and anyone (for testing)
  bvs_wallet = await DirectSecp256k1HdWallet.generate(12, {
    prefix: "wasm",
    hdPaths: [
      stringToPath("m/0"),
      stringToPath("m/1"),
      stringToPath("m/2"),
      stringToPath("m/3"),
      stringToPath("m/4"),
      stringToPath("m/5"),
    ],
  });
  const [owner, operator, staker, committeeMember_1, committeeMember_2, committeeMember_3] =
    await bvs_wallet.getAccounts();

  // Fund all 3 accounts with some tokens
  await started.fund(
    "10000000ustake",
    owner.address,
    operator.address,
    staker.address,
    committeeMember_1.address,
    committeeMember_2.address,
    committeeMember_3.address,
  );

  // Create a new signer with the wallet using the container as the RPC endpoint
  satlayer_wallet = started.wallet;
  clientSigner = await started.newSigner(bvs_wallet);

  // setup committee
  committee = [
    { addr: committeeMember_1.address, weight: 1 },
    { addr: committeeMember_2.address, weight: 1 },
    { addr: committeeMember_3.address, weight: 1 },
  ];

  const instantiated = await deployGovernanceContract(owner.address, committee);

  governanceContractAddress = instantiated.contractAddress;

  let register_as_operator: RegistryExecuteMsg = {
    register_as_operator: {
      metadata: {
        name: "Bad Operator",
      },
    },
  };

  let response = await clientSigner.execute(operator.address, contracts.registry.address, register_as_operator, "auto");

  expect(response).toBeDefined();

  let register_service_to_operator: RegistryExecuteMsg = {
    register_service_to_operator: {
      service: governanceContractAddress,
    },
  };

  response = await clientSigner.execute(
    operator.address,
    contracts.registry.address,
    register_service_to_operator,
    "auto",
  );

  expect(response).toBeDefined();

  // let's vote on if the operator should be registered to the service
  let action: RegistryExecuteMsg = {
    register_operator_to_service: {
      operator: operator.address,
    },
  };

  let proposal: GovernanceExecuteMsg = {
    base: {
      propose: {
        title: "Register Operator to Service",
        description: "Proposal to register operator to service",
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: contracts.registry.address,
                msg: Buffer.from(JSON.stringify(action)).toString("base64"),
                funds: [],
              },
            },
          },
        ],
      },
    },
  };

  response = await clientSigner.execute(committee[0].addr, governanceContractAddress, proposal, "auto");

  expect(response).toBeDefined();

  let proposal_id = response.events
    ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
    ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

  // the first proposal was enable slashing
  expect(proposal_id).toBe("1");

  // vote on the proposal
  // skip the first member as they are the proposer
  for (let i = 1; i < committee.length; i++) {
    let vote: GovernanceExecuteMsg = {
      base: {
        vote: {
          proposal_id: parseInt(proposal_id as string),
          vote: "yes" as Vote,
        },
      },
    };
    response = await clientSigner.execute(committee[i].addr, governanceContractAddress, vote, "auto");
    expect(response).toBeDefined();
  }

  // execute the proposal
  let execute: GovernanceExecuteMsg = {
    base: {
      execute: {
        proposal_id: parseInt(proposal_id as string),
      },
    },
  };
  response = await clientSigner.execute(committee[0].addr, governanceContractAddress, execute, "auto");
  expect(response).toBeDefined();

  let register_status: RegistryQueryMsg = {
    status: {
      operator: operator.address,
      service: governanceContractAddress,
    },
  };

  let status_response = await clientSigner.queryContractSmart(contracts.registry.address, register_status);

  expect(status_response).toBe(1); // 1 means Active

  vaultAddress = await contracts.initVaultBankTokenized(operator.address, "ustake");

  api = new Api({
    client: clientSigner,
    vault: vaultAddress,
    registry: contracts.registry.address,
    router: contracts.router.address,
    service: governanceContractAddress,
    rewards: contracts.rewards.address,
  });
}, 60_000);

afterAll(async () => {
  await started.stop();
});

test("Rewards Lifecycle", async () => {
  await setup_staking();
  offChainRewardTrigger(api, (root, data) => console.log("Reward Triggered", root, data));
});
