// bvs.pl.cg.lifecycle.test.ts
import { beforeAll, expect, test, vi } from "vitest";
import { AnvilContainer, ChainName, EVMContracts, StartedAnvilContainer } from "@satlayer/testcontainers";
import { Account, encodeAbiParameters, getContract, parseAbi, parseEther, parseUnits, Hex, toHex } from "viem";

import {
  abi as bvsAbi,
  bytecode as bvsBytecode,
} from "../out/StablecoinCollateralBVS.sol/StablecoinCollateralBVS.json";
import { abi as vaultAbi } from "@satlayer/contracts/out/SLAYVaultV2.sol/SLAYVaultV2.json";
import { abi as mockErc20Abi } from "@satlayer/contracts/out/MockERC20.sol/MockERC20.json";

import { abi as plAbi } from "../out/PositionLocker.sol/PositionLocker.json";
import { abi as cgAbi, bytecode as cgBytecode } from "../out/ConversionGateway.sol/ConversionGateway.json";
import { abi as evcAbi, bytecode as evcBytecode } from "../out/ExternalVaultConnector.sol/ExternalVaultConnector.json";
import { abi as s4626Abi, bytecode as s4626Bytecode } from "../out/ExternalVaultConnector.t.sol/Simple4626.json";
import { abi as oracleAbi } from "../out/ConversionGateway.t.sol/MockOracle.json";
import { bytecode as oraclebytecode } from "../out/ConversionGateway.t.sol/MockOracle.json";
import {
  abi as adapterAbi,
  bytecode as adapterBytecode,
} from "../out/StablecoinFull.t.sol/MockBorrowVenueAdapter.json";
import { bytecode as plbytecode } from "../out/PositionLocker.sol/PositionLocker.json";

import { OperatorNode } from "../src/operatorNode";

// util to compute function selector
const selector = (
  sig: string, // keccak256(sig).slice(0,10)
) => ("0x" + require("js-sha3").keccak_256(sig).slice(0, 8)) as Hex;

function asBytes(x?: Hex | Uint8Array | string | null): Hex {
  if (x == null) return "0x";
  if (typeof x === "string") return (x === "0x" ? "0x" : x.startsWith("0x") ? x : toHex(x)) as Hex;
  // Uint8Array or Buffer
  return toHex(x);
}

/* ========================= Mine helpers ========================= */

async function mineUntilReceipt(
  eth: StartedAnvilContainer,
  client: ReturnType<StartedAnvilContainer["getClient"]>,
  hash: Hex,
  maxTries = 10,
) {
  for (let i = 0; i < maxTries; i++) {
    await eth.mineBlock(1);
    const r = await client.getTransactionReceipt({ hash }).catch(() => null);
    if (r) return r;
  }
  throw new Error(`Receipt not found after ${maxTries} mined blocks: ${hash}`);
}

async function sendAndMine(
  eth: StartedAnvilContainer,
  client: ReturnType<StartedAnvilContainer["getClient"]>,
  txHashPromise: Promise<Hex>,
) {
  const hash = await txHashPromise; // broadcasted but pending
  await eth.mineBlock(1);
  const receipt = await client.getTransactionReceipt({ hash });
  if (receipt.status !== "success") throw new Error(`Tx failed: ${hash}`);
  return receipt;
}

type CanFinalizeResult = readonly [boolean, bigint, readonly bigint[], bigint];

let eth: StartedAnvilContainer;
let contracts: EVMContracts;

let owner: Account;
let operator: Account;
let alice: Account;

let vault: `0x${string}`;
let BASE: `0x${string}`;
let DEBT: `0x${string}`;

let bvs: Awaited<ReturnType<typeof eth.deployContract>>;
let pl: Awaited<ReturnType<typeof eth.deployContract>>;
let cg: Awaited<ReturnType<typeof eth.deployContract>>;
let adapter: Awaited<ReturnType<typeof eth.deployContract>>;

let extBorrow: Awaited<ReturnType<typeof eth.deployContract>>;
let connBorrow: Awaited<ReturnType<typeof eth.deployContract>>;
let oracle: Awaited<ReturnType<typeof eth.deployContract>>;

let node: OperatorNode;

const STRAT_BORROW = "0x" + require("js-sha3").keccak_256("ROUTE_WRAP");
const STRAT_BORROW_ID = STRAT_BORROW as `0x${string}`;

beforeAll(async () => {
  // chain + infra
  eth = await new AnvilContainer({ forkedChainName: ChainName.EthereumMainnet }).start();
  contracts = await EVMContracts.bootstrap(eth);
  await contracts.initOracle();

  owner = eth.getAccount() as Account;
  operator = eth.generateAccount("operator") as Account;
  alice = eth.generateAccount("alice") as Account;

  await eth.setBalance(owner.address, parseEther("1"));
  await eth.setBalance(operator.address, parseEther("1"));
  await eth.setBalance(alice.address, parseEther("1"));
  await eth.mineBlock(1);

  // register operator at SatLayer
  await contracts.registry.write.registerAsOperator(["https://operator", "Operator Y"], { account: operator });
  await eth.mineBlock(1);

  // tokens
  const base = await contracts.initERC20({ name: "WBTC", symbol: "WBTC", decimals: 8 });
  const debt = await contracts.initERC20({ name: "wWBTC", symbol: "wWBTC", decimals: 8 });
  BASE = base.contractAddress;
  DEBT = debt.contractAddress;

  // vault by operator
  vault = await contracts.initVault({ operator, underlyingAsset: BASE });
  await contracts.router.write.setVaultWhitelist([vault, true]);
  await eth.mineBlock(1);

  // deploy PL

  const salt = "v1"; // o `pl-${Date.now()}` si no te importa la determinación

  pl = await eth.deployContract({
    abi: plAbi,
    bytecode: plbytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [vault],
  });

  // oracle

  oracle = await eth.deployContract({
    abi: oracleAbi,
    bytecode: oraclebytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [],
  });
  const Oracle = getContract({ address: oracle.contractAddress, abi: oracleAbi, client: eth.getClient() });
  await Oracle.write.set([BASE, 50_000n * 10n ** 8n]); // 50k*1e8
  await Oracle.write.set([DEBT, 10n ** 8n]); // 50k*1e8

  await eth.mineBlock(1);

  // CG
  cg = await eth.deployContract({
    abi: cgAbi,
    bytecode: cgBytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [
      owner.address,
      operator.address,
      operator.address,
      pl.contractAddress,
      oracle.contractAddress,
      BASE,
    ],
  });

  // adapter

  adapter = await eth.deployContract({
    abi: adapterAbi,
    bytecode: adapterBytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [BASE, DEBT],
  });

  // ext 4626 (asset = DEBT) and connector

  extBorrow = await eth.deployContract({
    abi: s4626Abi,
    bytecode: s4626Bytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [DEBT, "ext vWBTC", "ext-vWBTC"],
  });

  connBorrow = await eth.deployContract({
    abi: evcAbi,
    bytecode: evcBytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [owner.address, cg.contractAddress, extBorrow.contractAddress],
  });

  // configure strategy on CG
  const CG = getContract({ address: cg.contractAddress, abi: cgAbi, client: eth.getClient() });
  const sDeposit = {
    wrapper: "0x0000000000000000000000000000000000000000",
    connector: "0x0000000000000000000000000000000000000000",
    safety: {
      redeemToleranceBps: 25,
      unwrapMinOutBps: 9950,
      emergencyMode: false,
      emergencyRedeemBps: 500,
      emergencyUnwrapBps: 500,
    },
  };
  const sBorrow = {
    adapter: adapter.contractAddress,
    debtAsset: debt.contractAddress,
    borrowedConnector: connBorrow.contractAddress,
    maxBorrowBps: 7000,
    safety: {
      redeemToleranceBps: 50,
      withdrawSlippageBps: 50,
      maxAprBps: 1500,
      minHfBps: 1200,
      emergencyMode: false,
      emergencyRedeemBps: 300,
      emergencyWithdrawBps: 300,
    },
  };
  await CG.write.setStrategy([STRAT_BORROW_ID, { kind: 2, deposit: sDeposit, borrow: sBorrow }]); // kind=DepositWrap1to1
  await eth.mineBlock(1);

  // PL setup (caps, gateway, enable strategy, unpause)
  const PL = getContract({ address: pl.contractAddress, abi: plAbi, client: eth.getClient() });
  await PL.write.setCaps([5000, 5000, 5000, 86400], { account: operator });
  await PL.write.setConversionGateway([cg.contractAddress], { account: operator });
  await PL.write.setStrategyEnabled([STRAT_BORROW_ID, true], { account: operator });
  const psHash = await PL.write.setPaused([false], { account: operator.address });
  await mineUntilReceipt(eth, eth.getClient(), psHash as any);

  // deploy BVS + register operator

  bvs = await eth.deployContract({
    abi: bvsAbi,
    bytecode: bvsBytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [contracts.registry.address, owner.address],
  });

  await bvs.contract.write.registerOperator([operator.address], { account: owner });
  await contracts.registry.write.registerServiceToOperator([bvs.contractAddress], { account: operator });
  await eth.mineBlock(1);

  // Node
  const node = new OperatorNode(
    "OP",
    eth.getClient(),
    eth, // StartedAnvilContainer
    operator, // Account
    bvs.contractAddress,
  );

  node.start();
}, 180_000);

test("BVS-driven: claimTo then unwindBorrow (borrow path) -> fully unlockable", async () => {
  const client = eth.getClient();
  const Vault = getContract({ address: vault, abi: vaultAbi, client });
  const PL = getContract({ address: pl.contractAddress, abi: plAbi, client });
  const CG = getContract({ address: cg.contractAddress, abi: cgAbi, client });
  const BVS = getContract({ address: bvs.contractAddress, abi: bvsAbi, client });
  const BASEc = getContract({ address: BASE, abi: mockErc20Abi, client });
  const DEBTc = getContract({ address: DEBT, abi: mockErc20Abi, client });
  const Conn = getContract({ address: connBorrow.contractAddress, abi: evcAbi, client });

  // fund alice & deposit to vault, then opt-in
  const depositAmt = parseUnits("500", 8);

  // 1) mint/approve/deposit
  await BASEc.write.mint([alice.address, parseUnits("500", 8)]);
  await eth.mineBlock(1);

  const approveBaseHash = await BASEc.write.approve([Vault.address, depositAmt], { account: alice });
  await eth.mineBlock(1);

  const depositHash = await Vault.write.deposit([parseUnits("500", 8), alice.address], { account: alice });
  await mineUntilReceipt(eth, client, depositHash as any);

  // 2) opt-in
  const balShares = (await getContract({
    address: vault,
    abi: parseAbi(["function balanceOf(address) view returns (uint256)"]),
    client,
  }).read.balanceOf([alice.address])) as bigint;

  // Approve PL to transfer Vault shares
  const approveSharesHash = await Vault.write.approve([PL.address, balShares], { account: alice });
  await mineUntilReceipt(eth, eth.getClient(), approveSharesHash as any);

  const psHash = await PL.write.setPaused([false], { account: operator.address });
  await mineUntilReceipt(eth, eth.getClient(), psHash as any);

  // struct StrategyId is bytes32; PL.optIn(shares, StrategyId)
  const txoptin = await PL.write.optIn([balShares, STRAT_BORROW_ID], { account: alice });
  await mineUntilReceipt(eth, eth.getClient(), txoptin as any);

  // operator creates a PL requestFor (same as Foundry test)
  const reqShares = balShares / 2n;
  const reqSim = await PL.simulate.requestFor([alice.address, reqShares, STRAT_BORROW_ID], {
    account: operator.address,
  });

  const reqId = reqSim.result as bigint;
  const txrequestFor = await PL.write.requestFor([alice.address, reqShares, STRAT_BORROW_ID], { account: operator });
  await mineUntilReceipt(eth, eth.getClient(), txrequestFor as any);

  // make claimable

  await client.request({ method: "evm_increaseTime", params: [toHex(7 * 24 * 60 * 60)] });

  await mineUntilReceipt(eth, eth.getClient(), txrequestFor as any);

  // === BVS opens a two 1 action request: claimTo(reqId,""), unwindDepositAny(alice, STRAT_BORROW, max)
  const chainId = BigInt(await client.getChainId());
  const CLAIM_TO = selector("claimTo(uint256,bytes)");
  //const UNWIND_ANY = selector("unwindDepositAny(address,bytes32,uint256)");

  const UNWIND_ANY = selector("unwindBorrow(address,bytes32,uint256,uint256,bytes,uint256)");

  const params = encodeAbiParameters(
    [{ type: "uint16" }, { type: "bytes" }, { type: "bytes" }, { type: "uint256" }],
    [5000, "0x", "0x", 0n],
  );

  const claimPrefix = encodeAbiParameters(
    [{ type: "uint256" }, { type: "bytes" }], // only reqId as prefix
    [reqId, params as Hex],
  );

  const maxVal = BigInt(2 ** 256) - BigInt(1);

  const minCollateralOut = BigInt(1); // accept any positive base after rounding shaves

  const unwindPrefix = encodeAbiParameters(
    [
      { type: "address" },
      { type: "bytes32" },
      { type: "uint256" },
      { type: "uint256" },
      { type: "bytes" },
      { type: "uint256" },
    ],

    [alice.address, STRAT_BORROW_ID, maxVal, minCollateralOut, asBytes(), 1n],
  );

  const expectedArgsHash = "0x" + require("js-sha3").keccak_256(claimPrefix);
  const expectedArgsHash2 = "0x" + require("js-sha3").keccak_256(unwindPrefix);

  const actions: readonly [
    `0x${string}`, // target
    Hex, // selector (bytes4)
    Hex, // expectedArgs (bytes)
    Hex, // extraData (bytes)
    Hex,
    number, // matchMode (uint8)
  ][] = [
    [
      pl.contractAddress as `0x${string}`,
      CLAIM_TO, // CLAIM_TO selector (bytes4)
      asBytes(claimPrefix), // expectedArgs
      expectedArgsHash as `0x${string}`,
      "0x", // extraData
      2, // PREFIX
    ],
  ];

  // call openRequest
  const completionAll = 1; // CompletionMode.ALL
  const kRequired = 0;
  const quorumBps = 0;
  const minCount = 1;
  const ttlSeconds = 3600;
  const operatorAllow = [] as `0x${string}`[];

  const txSim = await BVS.simulate.openRequest(
    [chainId, actions, completionAll, kRequired, quorumBps, minCount, ttlSeconds, operatorAllow],
    { account: owner.address },
  );

  const bvsReqId = txSim.result as bigint;
  const txopen = await BVS.write.openRequest([chainId, actions, 1, 0, 0, minCount, 3600, []], { account: owner });
  await mineUntilReceipt(eth, eth.getClient(), txopen as any);

  // wait for node to execute  action & attest, then finalize
  await vi.waitFor(
    async () => {
      const can = (await bvs.contract.read.canFinalize([bvsReqId])) as CanFinalizeResult;
      const ok = Boolean(can[0]);
      if (ok) {
        const fintx = await bvs.contract.write.finalizeRequest([bvsReqId], { account: owner });
        await mineUntilReceipt(eth, eth.getClient(), fintx as any);
      }
      expect(ok).toBe(true);
    },
    { interval: 500, timeout: 30_000 },
  );

  //Simulating yield to the vault

  const entitlement = (await Conn.read.assetsOf([alice.address])) as bigint;

  const yield_value = entitlement / BigInt(10); // +10%

  const target_vault = await Conn.read.targetVault();

  const mintTx = await DEBTc.write.mint([target_vault, yield_value]);
  await mineUntilReceipt(eth, eth.getClient(), mintTx as any);

  const entitlement_after = (await Conn.read.assetsOf([alice.address])) as bigint;

  expect(entitlement_after).toBeGreaterThan(entitlement);

  const actions2: readonly [
    `0x${string}`, // target
    Hex, // selector (bytes4)
    Hex, // expectedArgs (bytes)
    Hex, // extraData (bytes)
    Hex,
    number, // matchMode (uint8)
  ][] = [
    [
      cg.contractAddress as `0x${string}`,
      UNWIND_ANY, // UNWIND_ANY selector (bytes4)
      asBytes(unwindPrefix), // expectedArgs
      expectedArgsHash2 as `0x${string}`,
      "0x", // extraData
      2,
    ],
  ];

  const txSimreq2 = await BVS.simulate.openRequest(
    [chainId, actions2, completionAll, kRequired, quorumBps, minCount, ttlSeconds, operatorAllow],
    { account: owner.address },
  );

  //const txSim = await bvs.contract.simulate.openRequest([chainId, actions, /*ALL*/ 1, /*k*/0, /*bps*/0, minCount, /*ttl*/3600, []], { account: owner });
  const bvsReqId2 = txSimreq2.result as bigint;
  const txopen2 = await BVS.write.openRequest([chainId, actions2, 1, 0, 0, minCount, 3600, []], { account: owner });

  await mineUntilReceipt(eth, eth.getClient(), txopen2 as any);

  // wait for node to execute  action & attest, then finalize

  await vi.waitFor(
    async () => {
      const can = (await bvs.contract.read.canFinalize([bvsReqId2])) as CanFinalizeResult;
      const ok = Boolean(can[0]);
      if (ok) {
        const fintx2 = await bvs.contract.write.finalizeRequest([bvsReqId2], { account: owner });
        await mineUntilReceipt(eth, eth.getClient(), fintx2 as any);
      }
      expect(ok).toBe(true);
    },
    { interval: 500, timeout: 30_000 },
  );

  // Assertions
  // connector entitlement should be 0 after full unwind

  const ent = (await Conn.read.assetsOf([alice.address])) as bigint;

  expect(ent).toBe(0n);

  // user should be fully unlockable for the strategy
  const unlockable = (await PL.read.unlockable([alice.address, STRAT_BORROW_ID])) as bigint;
  const totals = (await PL.read.userTotals([alice.address])) as any[];
  const totalShares2 = totals[0] as bigint;

  expect(unlockable).toBe(totalShares2);

  // user can opt-out all
  const outputtx = await PL.write.optOutAll([[STRAT_BORROW_ID]], { account: alice });
  await mineUntilReceipt(eth, eth.getClient(), outputtx as any);

  // sanity: vaulted share balance returned to user
  const vBal = (await getContract({
    address: vault,
    abi: parseAbi(["function balanceOf(address) view returns (uint256)"]),
    client,
  }).read.balanceOf([alice.address])) as bigint;

  expect(vBal).toBe(totalShares2);
}, 120_000);
