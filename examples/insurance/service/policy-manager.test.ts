import { beforeAll, describe, expect, test } from "vitest";
import { PolicyManager } from "./policy-manager";
import { Api } from "./api";
import { CosmWasmContainer, SatLayerContracts, StartedCosmWasmContainer } from "@satlayer/testcontainers";
import { AccountData, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { stringToPath } from "@cosmjs/crypto";
import { ExecuteMsg as VaultBankExecuteMsg } from "@satlayer/cosmwasm-schema/vault-bank";
import { ExecuteMsg as RegistryExecuteMsg } from "@satlayer/cosmwasm-schema/registry";
import { coins } from "@cosmjs/stargate";
import { sleep } from "@cosmjs/utils";
import { ExecuteMsg as GuardrailExecuteMsg } from "@satlayer/cosmwasm-schema/guardrail";

let policyManager: PolicyManager;
let api: Api;
let started: StartedCosmWasmContainer;
let contracts: SatLayerContracts;
let wallet: DirectSecp256k1HdWallet;

let vaultAddress: string;
let serviceAccount: AccountData;
let operatorAccount: AccountData;
let ownerAccount: AccountData;

describe("PolicyManager", () => {
  beforeAll(async () => {
    // Set up CosmWasmContainer with SatLayerContracts bootstrapped
    started = await new CosmWasmContainer().start();
    contracts = await SatLayerContracts.bootstrap(started);

    // A wallet with 3 accounts operator, staker and service
    wallet = await DirectSecp256k1HdWallet.generate(12, {
      prefix: "wasm",
      hdPaths: [stringToPath("m/0"), stringToPath("m/1"), stringToPath("m/2")],
    });
    const [operator, staker, service] = await wallet.getAccounts();

    // get owner from container
    const [owner] = await started.wallet.getAccounts();

    serviceAccount = service;
    operatorAccount = operator;
    ownerAccount = owner;

    // Fund all 4 accounts with some tokens
    await started.fund("10000000ustake", owner.address, operator.address, staker.address, service.address);

    // Init vault bank for operator
    vaultAddress = await contracts.initVaultBank(operator.address, "ustake");

    // Initialize Client
    let clientSigner = await started.newSigner(wallet);
    api = new Api({
      client: clientSigner,
      vault: vaultAddress,
      registry: contracts.registry.address,
      router: contracts.router.address,
      operator: operator.address,
      service: service.address,
    });

    // Operator register into Registry
    let registerOperatorMsg: RegistryExecuteMsg = {
      register_as_operator: {
        metadata: {
          name: "Operator",
          uri: "https://operator.com",
        },
      },
    };
    await clientSigner.execute(operator.address, contracts.registry.address, registerOperatorMsg, "auto");

    // Staker stake into vault bank
    let stakeMsg: VaultBankExecuteMsg = {
      deposit_for: {
        amount: "10000", // stake 10_000 ustake
        recipient: staker.address,
      },
    };
    await clientSigner.execute(staker.address, vaultAddress, stakeMsg, "auto", null, coins(10_000, "ustake"));

    // Initialize PolicyManager
    policyManager = new PolicyManager(api, serviceAccount.address, operator.address);
    await policyManager.init();

    // Register Service to Operator
    let registerServiceToOperatorMsg: RegistryExecuteMsg = {
      register_service_to_operator: {
        service: serviceAccount.address,
      },
    };
    await clientSigner.execute(operator.address, contracts.registry.address, registerServiceToOperatorMsg, "auto");
  }, 60_000);

  test("Lifecycle test", { timeout: 60_000 }, async () => {
    await sleep(500);
    // Alice buys policy for 1000 coverage
    let alice = await started.generateAccount("alice");
    let res = await policyManager.buyPolicy(1000, alice.address);

    expect(res).toStrictEqual({
      id: 1,
      insuree: alice.address,
      coverage: 1000,
      premium: 5,
      boughtAt: expect.any(Number),
      expiryAt: expect.any(Number),
    });

    await sleep(1000);

    // Alice proceeds to claim
    let claimRes = await policyManager.claimPolicy(res.id);
    // expect claimRes to be slashing request id with 64 hex chars (32 bytes)
    expect(claimRes).toMatch(/^[0-9a-fA-F]{64}$/);

    // propose to guardrail
    let proposeMsg: GuardrailExecuteMsg = {
      propose: {
        reason: "Payout",
        slashing_request_id: claimRes,
      },
    };
    await started.client.execute(ownerAccount.address, contracts.guardrail.address, proposeMsg, "auto");

    // proceed to process claim
    let processClaimRes = await policyManager.processClaim(claimRes);

    expect(processClaimRes).toStrictEqual({
      policyDetails: expect.any(Object),
      txHash: expect.any(String),
      payout: 1000,
      payoutAt: expect.any(Number),
    });

    // check alice balance
  });
});
