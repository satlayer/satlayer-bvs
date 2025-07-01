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
import { Committee } from "./committee";

let started: StartedCosmWasmContainer;
let contracts: SatLayerContracts;
let bvs_wallet: DirectSecp256k1HdWallet;
let satlayer_wallet: DirectSecp256k1HdWallet;
let clientSigner: SigningCosmWasmClient;
let governanceContractAddress: string;
let committee: Committee;
let vaultBankAddress: string;

async function deployGovernanceContract(owner: string, members: Voter[]) {
  const contractPath = require.resolve("@examples/governance-contract/dist/contract.wasm");
  const uploaded = await clientSigner.upload(owner, await readFile(contractPath), "auto");
  const initMsg: InstantiateMsg = {
    registry: contracts.registry.address,
    router: contracts.router.address,
    owner: owner,
    cw3_instantiate_msg: {
      voters: members,
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

async function satlayerGuardrailApprove(slashingRequestId: string) {
  let satlayer_signers_client = await started.newSigner(satlayer_wallet);

  let msg: GuardrailExecuteMsg = {
    propose: {
      slashing_request_id: slashingRequestId,
      reason: "Finalize Slashing Approval",
    },
  };

  let [owner] = await satlayer_wallet.getAccounts();

  let response = await satlayer_signers_client.execute(owner.address, contracts.guardrail.address, msg, "auto");
  expect(response).toBeDefined();
}

async function committeeEnableSlashing() {
  let action: RegistryExecuteMsg = {
    enable_slashing: {
      slashing_parameters: {
        max_slashing_bips: 5000, // 50%
        destination: governanceContractAddress,
        resolution_window: 5,
      },
    },
  };

  let response = await committee.propose(
    action,
    "Enable Slashing",
    "Proposal to enable slashing for the service",
    contracts.registry.address,
  );

  let proposal_id = response.events
    ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
    ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

  await committee.allVoteYes(parseInt(proposal_id as string));

  response = await committee.executeProposal(parseInt(proposal_id as string));

  let query: RegistryQueryMsg = {
    slashing_parameters: {
      service: governanceContractAddress,
    },
  };

  let slashing_parameters = (await clientSigner.queryContractSmart(
    contracts.registry.address,
    query,
  )) as SlashingParametersResponse;

  expect(slashing_parameters).toBeDefined();
}

async function setupStaking() {
  let [bvs_owner, _operator, staker] = await bvs_wallet.getAccounts();

  let msg: VaultBankExecuteMsg = {
    deposit_for: {
      recipient: staker.address,
      amount: "3000",
    },
  };

  let coin: Coin = {
    denom: "ustake",
    amount: "3000",
  };

  return clientSigner.execute(bvs_owner.address, vaultBankAddress, msg, "auto", undefined, [coin]);
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
  let multi_sig_members = [
    { addr: committeeMember_1.address, weight: 1 },
    { addr: committeeMember_2.address, weight: 1 },
    { addr: committeeMember_3.address, weight: 1 },
  ];

  const instantiated = await deployGovernanceContract(owner.address, multi_sig_members);

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

  committee = new Committee({
    client: clientSigner,
    vault: vaultBankAddress,
    registry: contracts.registry.address,
    router: contracts.router.address,
    service: governanceContractAddress,
    rewards: contracts.rewards.address,
    committee: multi_sig_members,
  });

  // let's vote on if the operator should be registered to the service
  let action: RegistryExecuteMsg = {
    register_operator_to_service: {
      operator: operator.address,
    },
  };

  response = await committee.propose(
    action,
    "Register Operator to Service",
    "Proposal to register operator to the service",
    contracts.registry.address,
  );

  let proposal_id = response.events
    ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
    ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

  await committee.allVoteYes(parseInt(proposal_id as string));

  response = await committee.executeProposal(parseInt(proposal_id as string));

  expect(response).toBeDefined();

  let register_status: RegistryQueryMsg = {
    status: {
      operator: operator.address,
      service: governanceContractAddress,
    },
  };

  let status_response = await clientSigner.queryContractSmart(contracts.registry.address, register_status);

  expect(status_response).toBe(1); // 1 means Active

  await committeeEnableSlashing();

  // sleep for a couple of second to ensure the slashing parameters are set
  await new Promise((resolve) => setTimeout(resolve, 2000));

  let operator_opt_in_slashing: RegistryExecuteMsg = {
    operator_opt_in_to_slashing: {
      service: governanceContractAddress,
    },
  };
  response = await clientSigner.execute(operator.address, contracts.registry.address, operator_opt_in_slashing, "auto");
  expect(response).toBeDefined();

  vaultBankAddress = await contracts.initVaultBank(operator.address, "ustake");
}, 60_000);

afterAll(async () => {
  await started.stop();
});

test(
  "Social Committee based slashing lifecycle",
  async () => {
    await setupStaking();

    const [owner, operator, staker] = await bvs_wallet.getAccounts();
    let action: RouterExecuteMsg = {
      request_slashing: {
        bips: 500,
        metadata: {
          reason: "Test slashing request",
        },
        operator: operator.address,
        timestamp: (Date.now() * 1000000).toString(), // Current timestamp in nanoseconds
      },
    };
    let response = await committee.propose(
      action,
      "Request Slashing",
      "Proposal to request slashing for the operator",
      contracts.router.address,
    );

    expect(response).toBeDefined();

    let proposal_id = response.events
      ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
      ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

    await committee.allVoteYes(parseInt(proposal_id as string));

    response = await committee.executeProposal(parseInt(proposal_id as string));
    expect(response).toBeDefined();

    let query_msg: RouterQueryMsg = {
      slashing_request_id: {
        operator: operator.address,
        service: governanceContractAddress,
      },
    };

    let slashing_request_id = await clientSigner.queryContractSmart(contracts.router.address, query_msg);

    query_msg = {
      slashing_request: slashing_request_id,
    };

    let slashing_request: SlashingRequestResponse = (await clientSigner.queryContractSmart(
      contracts.router.address,
      query_msg,
    )) as SlashingRequestResponse;

    expect(slashing_request.status).toBe(0);
    expect(slashing_request.request.bips).toBe(500);
    expect(slashing_request.request.operator).toBe(operator.address);
    expect(slashing_request.service).toBe(governanceContractAddress);

    // propose to lock the slash
    let lock_slash_action: RouterExecuteMsg = {
      lock_slashing: slashing_request_id,
    };

    response = await committee.propose(
      lock_slash_action,
      "Lock Slashing",
      "Proposal to lock slashing for the operator",
      contracts.router.address,
    );

    // now committee members vote on the proposal
    let lock_proposal_id = response.events
      ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
      ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

    await committee.allVoteYes(parseInt(lock_proposal_id as string));

    expect(response).toBeDefined();

    // sleep abit to let resolution window pass
    await new Promise((resolve) => setTimeout(resolve, 6000));

    response = await committee.executeProposal(parseInt(lock_proposal_id as string));

    // Collateral are locked in the router contract
    let router_balance = await clientSigner.getBalance(contracts.router.address, "ustake");
    expect(router_balance.amount).toBe("150");

    // Let's start finalizing the slashing request that would move the funds to the governance contract
    let finalize_slash_action: RouterExecuteMsg = {
      finalize_slashing: slashing_request_id,
    };

    response = await committee.propose(
      finalize_slash_action,
      "Finalize Slashing",
      "Proposal to finalize slashing for the operator",
      contracts.router.address,
    );

    expect(response).toBeDefined();

    let finalize_proposal_id = response.events
      ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
      ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

    await committee.allVoteYes(parseInt(finalize_proposal_id as string));

    // satlayer guardrail authorize the slashing finalization
    await satlayerGuardrailApprove(slashing_request_id as string);

    response = await committee.executeProposal(parseInt(finalize_proposal_id as string));
    expect(response).toBeDefined();

    let governance_balance = await clientSigner.getBalance(governanceContractAddress, "ustake");
    expect(governance_balance.amount).toBe("150");
  },
  60 * 1000,
);
