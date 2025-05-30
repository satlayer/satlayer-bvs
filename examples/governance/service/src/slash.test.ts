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

let started: StartedCosmWasmContainer;
let contracts: SatLayerContracts;
let bvs_wallet: DirectSecp256k1HdWallet;
let satlayer_wallet: DirectSecp256k1HdWallet;
let clientSigner: SigningCosmWasmClient;
let governanceContractAddress: string;
let committee: Voter[];
let vaultBankAddress: string;

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

async function enableSlashing() {
  let action: RegistryExecuteMsg = {
    enable_slashing: {
      slashing_parameters: {
        max_slashing_bips: 5000, // 50%
        destination: governanceContractAddress,
        resolution_window: 5,
      },
    },
  };

  let proposal: GovernanceExecuteMsg = {
    base: {
      propose: {
        title: "Enable Slashing",
        description: "Proposal to enable slashing for operators",
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

  let response = await clientSigner.execute(committee[0].addr, governanceContractAddress, proposal, "auto");

  expect(response).toBeDefined();

  let proposal_id = response.events
    ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
    ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

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

async function setup_staking() {
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

  await enableSlashing();
  // sleep for a second to ensure the slashing parameters are set
  await new Promise((resolve) => setTimeout(resolve, 1000));

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

test("Hello World", async () => {
  process.stdout.write("Hello World\n");
  expect(governanceContractAddress).toBeDefined();
}, 1200);

test(
  "Social Committee based slashing lifecycle",
  async () => {
    await setup_staking();

    const [owner, operator, staker] = await bvs_wallet.getAccounts();
    let action: RouterExecuteMsg = {
      request_slashing: {
        bips: 500,
        metadata: {
          reason: "Test slashing request",
        },
        operator: operator.address,
        // offense happened yesterday
        timestamp: (Date.now() * 1000000).toString(), // Current timestamp in nanoseconds
      },
    };
    let proposal: GovernanceExecuteMsg = {
      base: {
        propose: {
          title: "Slashing Request",
          description: "Request to slash operator for misconduct",
          msgs: [
            {
              wasm: {
                execute: {
                  contract_addr: contracts.router.address,
                  msg: Buffer.from(JSON.stringify(action)).toString("base64"),
                  funds: [],
                },
              },
            },
          ],
        },
      },
    };

    let response = await clientSigner.execute(committee[0].addr, governanceContractAddress, proposal, "auto");

    expect(response).toBeDefined();

    let proposal_id = response.events
      ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
      ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

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

    // query
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

    let proposal_msg: GovernanceExecuteMsg = {
      base: {
        propose: {
          title: "Lock Slashing",
          description: "Proposal to lock slashing for operator",
          msgs: [
            {
              wasm: {
                execute: {
                  contract_addr: contracts.router.address,
                  msg: Buffer.from(JSON.stringify(lock_slash_action)).toString("base64"),
                  funds: [],
                },
              },
            },
          ],
        },
      },
    };

    response = await clientSigner.execute(committee[0].addr, governanceContractAddress, proposal_msg, "auto");

    // now committee members vote on the proposal
    let lock_proposal_id = response.events
      ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
      ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

    expect(lock_proposal_id).toBe("4");

    // vote on the proposal
    for (let i = 1; i < committee.length; i++) {
      let vote: GovernanceExecuteMsg = {
        base: {
          vote: {
            proposal_id: parseInt(lock_proposal_id as string),
            vote: "yes" as Vote,
          },
        },
      };
      response = await clientSigner.execute(committee[i].addr, governanceContractAddress, vote, "auto");
      expect(response).toBeDefined();
    }

    // execute the proposal
    let execute_lock: GovernanceExecuteMsg = {
      base: {
        execute: {
          proposal_id: parseInt(lock_proposal_id as string),
        },
      },
    };

    // sleep abit to let resolution window pass
    await new Promise((resolve) => setTimeout(resolve, 6000));

    response = await clientSigner.execute(committee[0].addr, governanceContractAddress, execute_lock, "auto");

    // collateral are locked in the router contract
    let router_balance = await clientSigner.getBalance(contracts.router.address, "ustake");
    expect(router_balance.amount).toBe("150");

    // let's start finalizing the slashing request that would move the funds to the governance contract
    let finalize_slash_action: RouterExecuteMsg = {
      finalize_slashing: slashing_request_id,
    };

    let finalize_proposal_msg: GovernanceExecuteMsg = {
      base: {
        propose: {
          title: "Finalize Slashing",
          description: "Proposal to finalize slashing for operator",
          msgs: [
            {
              wasm: {
                execute: {
                  contract_addr: contracts.router.address,
                  msg: Buffer.from(JSON.stringify(finalize_slash_action)).toString("base64"),
                  funds: [],
                },
              },
            },
          ],
        },
      },
    };

    response = await clientSigner.execute(committee[0].addr, governanceContractAddress, finalize_proposal_msg, "auto");
    expect(response).toBeDefined();

    let finalize_proposal_id = response.events
      ?.find((event) => event.type === "wasm" && event.attributes.some((attr) => attr.key === "proposal_id"))
      ?.attributes.find((attr) => attr.key === "proposal_id")?.value;

    expect(finalize_proposal_id).toBe("5");
    // vote on the proposal
    // skip the first member as they are the proposer
    for (let i = 1; i < committee.length; i++) {
      let vote: GovernanceExecuteMsg = {
        base: {
          vote: {
            proposal_id: parseInt(finalize_proposal_id as string),
            vote: "yes" as Vote,
          },
        },
      };
      response = await clientSigner.execute(committee[i].addr, governanceContractAddress, vote, "auto");
      expect(response).toBeDefined();
    }

    // execute the proposal
    let execute_finalize: GovernanceExecuteMsg = {
      base: {
        execute: {
          proposal_id: parseInt(finalize_proposal_id as string),
        },
      },
    };

    await satlayerGuardrailApprove(slashing_request_id as string);

    response = await clientSigner.execute(committee[0].addr, governanceContractAddress, execute_finalize, "auto");

    let governance_balance = await clientSigner.getBalance(governanceContractAddress, "ustake");
    expect(governance_balance.amount).toBe("150");
  },
  60 * 1000,
);
