// bvs.pl.cg.lifecycle.test.ts
import { afterAll, beforeAll, expect, test, vi } from "vitest";
import { AnvilContainer, ChainName, EVMContracts, StartedAnvilContainer } from "@satlayer/testcontainers";
import {
  Account,
  encodeAbiParameters,
  getContract,
  parseAbi,
  parseEther,
  parseUnits,
  getFunctionSelector,
  parseAbiItem,
  Hex,
  toHex,
} from "viem";

import { createPublicClient, http } from "viem";

import { anvil, mainnet } from "viem/chains";

import { stringToHex } from "viem";

import { createWalletClient } from "viem";
import type { Abi, Address } from "viem";

import {
  abi as bvsAbi,
  bytecode as bvsBytecode,
} from "../out/StablecoinCollateralBVS.sol/StablecoinCollateralBVS.json";
import { abi as vaultAbi } from "@satlayer/contracts/out/SLAYVaultV2.sol/SLAYVaultV2.json";
import { abi as mockErc20Abi } from "@satlayer/contracts/out/MockERC20.sol/MockERC20.json";

import { abi as plAbi } from "../out/PositionLocker.sol/PositionLocker.json";
import { abi as cgAbi, bytecode as cgBytecode } from "../out/ConversionGateway.sol/ConversionGateway.json";
import { abi as evcAbi, bytecode as evcBytecode } from "../out/ExternalVaultConnector.sol/ExternalVaultConnector.json";
import { abi as wrapAbi, bytecode as wrapBytecode } from "../out/ConversionGateway.t.sol/MockWrapper1to1.json";
import { abi as s4626Abi, bytecode as s4626Bytecode } from "../out/ExternalVaultConnector.t.sol/Simple4626.json";
import { abi as oracleAbi } from "../out/ConversionGateway.t.sol/MockOracle.json";
import { bytecode as oraclebytecode } from "../out/ConversionGateway.t.sol/MockOracle.json";
import {
  abi as adapterAbi,
  bytecode as adapterBytecode,
} from "../out/StablecoinFull.t.sol/MockBorrowVenueAdapter.json";
import { bytecode as plbytecode } from "../out/PositionLocker.sol/PositionLocker.json";

import { OperatorNode, TargetAbiRegistry } from "../src/operatorNode";
import type { HandlerCtx } from "../src/operatorNode";

// --- selectors we will call via BVS ---
const sel_claimTo = "0x" + Buffer.from("claimTo(uint256,bytes)").toString("hex").slice(0, 8); // compute properly or hardcode
const sel_unwindDepositAny =
  "0x" + Buffer.from("unwindDepositAny(address,bytes32,uint256)").toString("hex").slice(0, 8);

// util to compute function selector (safe, not node Buffer)
const selector = (
  sig: string, // keccak256(sig).slice(0,10)
) => ("0x" + require("js-sha3").keccak_256(sig).slice(0, 8)) as Hex;
function withTimeout<T>(p: Promise<T>, ms: number, label = "step"): Promise<T> {
  return Promise.race([
    p,
    new Promise<never>((_, rej) => setTimeout(() => rej(new Error(`⏰ timeout @ ${label} after ${ms}ms`)), ms)),
  ]) as Promise<T>;
}

function asBytes(x?: Hex | Uint8Array | string | null): Hex {
  if (x == null) return "0x";
  if (typeof x === "string") return (x === "0x" ? "0x" : x.startsWith("0x") ? x : toHex(x)) as Hex;
  // Uint8Array or Buffer
  return toHex(x);
}

function stepper() {
  let i = 0;
  return async <T>(label: string, p: Promise<T> | (() => Promise<T>)) => {
    const n = ++i;
    const t0 = Date.now();
    console.log(`\n[STEP ${n}] ${label}…`);
    const res = await (typeof p === "function" ? (p as any)() : p);
    console.log(`[STEP ${n}] done in ${Date.now() - t0}ms`);
    return res;
  };
}

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
  await eth.mineBlock(1); // <- IMPORTANT (no automine)
  const receipt = await client.getTransactionReceipt({ hash });
  if (receipt.status !== "success") throw new Error(`Tx failed: ${hash}`);
  return receipt;
}

class SimpleAbiRegistry implements TargetAbiRegistry {
  private map = new Map<`0x${string}`, any[]>();
  add(target: `0x${string}`, abi: any[]) {
    this.map.set(target, abi);
  }
  getAbi(target: `0x${string}`) {
    return this.map.get(target) ?? null;
  }
}

type Recipe = (args: {
  ctx: any; // HandlerCtx
  fnAbi: any; // ABI function item
  // helpers:
  encode: typeof encodeAbiParameters;
}) => Promise<Hex | null> | Hex | null;

type CanFinalizeResult = readonly [boolean, bigint, readonly bigint[], bigint];

let eth: StartedAnvilContainer;
let contracts: EVMContracts;

let owner: Account;
let operator: Account;
let alice: Account;

let vault: `0x${string}`;
let BASE: `0x${string}`;
let WRAPPED: `0x${string}`;

let bvs: Awaited<ReturnType<typeof eth.deployContract>>;
let pl: Awaited<ReturnType<typeof eth.deployContract>>;
let cg: Awaited<ReturnType<typeof eth.deployContract>>;
let wrapper: Awaited<ReturnType<typeof eth.deployContract>>;
let adapter: Awaited<ReturnType<typeof eth.deployContract>>;

let extWrapped: Awaited<ReturnType<typeof eth.deployContract>>;
let connWrapped: Awaited<ReturnType<typeof eth.deployContract>>;
let oracle: Awaited<ReturnType<typeof eth.deployContract>>;

let node: OperatorNode;

const STRAT_WRAP = "0x" + require("js-sha3").keccak_256("ROUTE_WRAP");
const STRAT_WRAP_ID = STRAT_WRAP as `0x${string}`;
const abiRegistry = new SimpleAbiRegistry();

beforeAll(async () => {
  // chain + infra
  eth = await new AnvilContainer({ forkedChainName: ChainName.EthereumMainnet }).start();
  contracts = await EVMContracts.bootstrap(eth);
  await contracts.initOracle();

  owner = eth.getAccount() as Account;
  operator = eth.generateAccount("operator") as Account;
  alice = eth.generateAccount("alice") as Account;

  const opWallet = createWalletClient({
    account: operator,
    chain: mainnet, // pon foundry/anvil si no viene seteada
    transport: http(),
  });

  await eth.setBalance(owner.address, parseEther("1"));
  await eth.setBalance(operator.address, parseEther("1"));
  await eth.setBalance(alice.address, parseEther("1"));
  await eth.mineBlock(1);

  // register operator at SatLayer
  await contracts.registry.write.registerAsOperator(["https://operator", "Operator Y"], { account: operator });
  await eth.mineBlock(1);

  // tokens
  const base = await contracts.initERC20({ name: "WBTC", symbol: "WBTC", decimals: 8 });
  const wrapped = await contracts.initERC20({ name: "wWBTC", symbol: "wWBTC", decimals: 8 });
  BASE = base.contractAddress;
  WRAPPED = wrapped.contractAddress;

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

  abiRegistry.add(pl.contractAddress, plAbi);

  // oracle

  oracle = await eth.deployContract({
    abi: oracleAbi,
    bytecode: oraclebytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [],
  });
  const Oracle = getContract({ address: oracle.contractAddress, abi: oracleAbi, client: eth.getClient() });
  await Oracle.write.set([BASE, 50_000n * 10n ** 8n]); // 50k*1e8
  await Oracle.write.set([WRAPPED, 10n ** 8n]); // 50k*1e8

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
  abiRegistry.add(cg.contractAddress, cgAbi);
  // wrapper (1:1)

  wrapper = await eth.deployContract({
    abi: wrapAbi,
    bytecode: wrapBytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [BASE, WRAPPED],
  });

  adapter = await eth.deployContract({
    abi: adapterAbi,
    bytecode: adapterBytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [BASE, WRAPPED],
  });

  // ext 4626 (asset = WRAPPED) and connector

  extWrapped = await eth.deployContract({
    abi: s4626Abi,
    bytecode: s4626Bytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [WRAPPED, "ext vWBTC", "ext-vWBTC"],
  });

  connWrapped = await eth.deployContract({
    abi: evcAbi,
    bytecode: evcBytecode.object as unknown as `0x${string}`,
    salt: salt,
    constructorArgs: [owner.address, cg.contractAddress, extWrapped.contractAddress],
  });

  // configure strategy on CG (wrap→deposit)
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
    debtAsset: wrapped.contractAddress,
    borrowedConnector: connWrapped.contractAddress,
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
  await CG.write.setStrategy([STRAT_WRAP_ID, { kind: 2, deposit: sDeposit, borrow: sBorrow }]); // kind=DepositWrap1to1
  await eth.mineBlock(1);

  // PL setup (caps, gateway, enable strategy, unpause)
  const PL = getContract({ address: pl.contractAddress, abi: plAbi, client: eth.getClient() });
  await PL.write.setCaps([5000, 5000, 5000, 86400], { account: operator });
  await PL.write.setConversionGateway([cg.contractAddress], { account: operator });
  await PL.write.setStrategyEnabled([STRAT_WRAP_ID, true], { account: operator });
  const psHash = await PL.write.setPaused([false], { account: operator.address });
  await mineUntilReceipt(eth, eth.getClient(), psHash as any);
  await eth.mineBlock(1);

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

  // 4) Registry mínimo de ABIs por target (ejemplo)
  type TargetAbiRegistry = Map<`0x${string}`, any>; // o un wrapper con get/set
  const registry: TargetAbiRegistry = new Map();
  // p.ej. si vas a invocar el ConversionGateway, registra su ABI:
  registry.set(cg.contractAddress as `0x${string}`, cgAbi);

  // 5) (Opcional) ParamResolver si tu BVS usa “expectedArgs” dinámicos
  const paramResolver = undefined;

  // 6) Construye el node correctamente:
  const node = new OperatorNode(
    "OP",
    eth.getClient(),
    eth, // StartedAnvilContainer
    operator, // Account
    bvs.contractAddress, // <- AQUÍ va la dirección del BVS
    abiRegistry, // <- AQUÍ va tu TargetAbiRegistry
  );

  // register handler for PL.claimTo(uint256,bytes)
  const CLAIM_TO = selector("claimTo(uint256,bytes)");
  //   node.onSelector(CLAIM_TO, async (ctx) => {
  //     const txHash = await ctx.operator.wallet.sendTransaction({
  //       to: ctx.action.target,
  //       data: (ctx.action.selector + ctx.action.expectedArgs.slice(2)) as Hex,
  //       account: ctx.operator.address,
  //     });
  //     return { txHash };
  //   });

  //     // Registrar handler para un selector concreto (p.ej. CLAIM_TO)
  //     node.addHandler(CLAIM_TO as Hex, async (ctx) => {
  //     const data = (ctx.selector + ctx.expectedArgs.slice(2)) as Hex; // selector || selector+args
  //     const txHash = await ctx.client.sendTransaction({
  //         account: ctx.operator,     // Account de viem (p.ej. privateKeyToAccount)
  //         to: ctx.target , // 0x...
  //         data: data ,     // 0x...
  //         chain:  anvil,
  //     });
  //     return { txHash };
  //     });

  // resolver.addSelectorOverrides(CLAIM_TO as Hex, {
  // // for PREFIX: append the bytes argument = "0x"
  // tail: async () => [ "0x" ],  // matches claimTo(uint256,bytes)
  // });

  // handler for CG.unwindDepositAny(address,bytes32,uint256)
  //const UNWIND_ANY = selector("unwindDepositAny(address,bytes32,uint256)");
  //   node.onSelector(UNWIND_ANY, async (ctx) => {
  //     const txHash = await ctx.operator.wallet.sendTransaction({
  //       to: ctx.action.target,
  //       data: (ctx.action.selector + ctx.action.expectedArgs.slice(2)) as Hex,
  //       account: ctx.operator.address,
  //     });
  //     return { txHash };

  //   });

  // resolver.addSelectorOverrides(UNWIND_ANY as Hex, {
  // // for PREFIX: append the bytes argument = "0x"
  // tail: async () => [ "0x" ],  // matches claimTo(uint256,bytes)
  // });

  node.start();
}, 180_000);

// afterAll(async () => {
//   node.stop();
//   await eth.stop();
// });

test("BVS-driven: claimTo then unwindDepositAny (wrap path) → fully unlockable", async () => {
  const client = eth.getClient();
  const step = stepper();

  const Vault = getContract({ address: vault, abi: vaultAbi, client });
  const PL = getContract({ address: pl.contractAddress, abi: plAbi, client });
  const CG = getContract({ address: cg.contractAddress, abi: cgAbi, client });
  const BVS = getContract({ address: bvs.contractAddress, abi: bvsAbi, client });
  const BASEc = getContract({ address: BASE, abi: mockErc20Abi, client });
  const Conn = getContract({ address: connWrapped.contractAddress, abi: evcAbi, client });

  // fund alice & deposit to vault, then opt-in
  const depositAmt = parseUnits("500", 8);

  // 1) mint/approve/deposit
  await step("mint base", BASEc.write.mint([alice.address, parseUnits("500", 8)]));
  await eth.mineBlock(1);

  const approveBaseHash = await BASEc.write.approve([Vault.address, depositAmt], { account: alice });
  await eth.mineBlock(1);

  const depositHash = await step(
    "deposit",
    Vault.write.deposit([parseUnits("500", 8), alice.address], { account: alice }),
  );
  await mineUntilReceipt(eth, client, depositHash as any);

  // 2) opt-in
  const balShares = (await getContract({
    address: vault,
    abi: parseAbi(["function balanceOf(address) view returns (uint256)"]),
    client,
  }).read.balanceOf([alice.address])) as bigint;

  //await sendAndMine(eth, client, PL.write.optIn([balShares, STRAT_WRAP_ID], { account: alice }));

  // // 1) Mint BASE to Alice (owner mints)
  // const mintHash = await BASEc.write.mint([alice.address, depositAmt], { account: owner });

  // // 2) Approve Vault to pull BASE
  // const approveBaseHash = await BASEc.write.approve([Vault.address, depositAmt], { account: alice });

  // // 3) Deposit BASE -> Vault (mints shares to Alice)
  // console.log('depositing');
  // const depositHash = await Vault.write.deposit([depositAmt, alice.address], { account: alice });
  // eth.mineBlock(20);
  // await step('wait mint', client.waitForTransactionReceipt({ hash: depositHash, timeout: 5 }));

  // await client.waitForTransactionReceipt({ hash: depositHash });

  // // 4) Read share balance AFTER deposit is mined
  // const balShares = await Vault.read.balanceOf([alice.address]) as bigint;
  // console.log('vault shares:', balShares.toString());
  // if (balShares === 0n) throw new Error('No shares minted — check deposit preconditions.');

  // 5) Approve PL to transfer Vault shares
  console.log("approving");
  const approveSharesHash = await Vault.write.approve([PL.address, balShares], { account: alice });
  //await client.waitForTransactionReceipt({ hash: approveSharesHash });
  await mineUntilReceipt(eth, eth.getClient(), approveSharesHash as any);
  eth.mineBlock(20);

  // await BASEc.write.mint([alice.address, depositAmt]);
  // await BASEc.write.approve([vault, depositAmt], { account: alice });
  // eth.mineBlock(1);

  // await Vault.write.deposit([depositAmt, alice.address], { account: alice });

  // eth.mineBlock(1);

  // //shares = vault.deposit(baseAssets, user);

  // // opt-in all shares to WRAP strategy
  // const balShares = await getContract({ address: vault, abi: parseAbi(["function balanceOf(address) view returns (uint256)"]), client })
  //   .read.balanceOf([alice.address]) as bigint;

  // console.log(balShares);

  // await getContract({ address: vault, abi: parseAbi(["function approve(address,uint256) returns (bool)"]), client })
  //   .write.approve([pl.contractAddress, balShares], {
  //   account: alice.address,
  //   gas: 300_000n,
  //   chain: mainnet
  // });
  const psHash = await PL.write.setPaused([false], { account: operator.address });
  await mineUntilReceipt(eth, eth.getClient(), psHash as any);

  console.log("optin");
  // struct StrategyId is bytes32 wrapper; PL.optIn(shares, StrategyId)
  const txoptin = await PL.write.optIn([balShares, STRAT_WRAP_ID], { account: alice });
  await mineUntilReceipt(eth, eth.getClient(), txoptin as any);

  // operator creates a PL requestFor (same as Foundry test)
  const reqShares = balShares / 2n;
  const reqSim = await PL.simulate.requestFor([alice.address, reqShares, STRAT_WRAP_ID], {
    account: operator.address,
    chain: mainnet,
  });

  console.log("reqSim.result:", reqSim.result);

  const reqId = reqSim.result as bigint;
  const txrequestFor = await PL.write.requestFor([alice.address, reqShares, STRAT_WRAP_ID], { account: operator });
  await mineUntilReceipt(eth, eth.getClient(), txrequestFor as any);

  // make claimable
  await eth.mineBlock(1);

  await client.request({ method: "evm_increaseTime", params: [toHex(7 * 24 * 60 * 60)] });

  await eth.mineBlock(1);
  await mineUntilReceipt(eth, eth.getClient(), txrequestFor as any);

  // === BVS opens a two-action request: claimTo(reqId,""), unwindDepositAny(alice, STRAT_WRAP, max)
  const chainId = BigInt(await client.getChainId());
  const CLAIM_TO = selector("claimTo(uint256,bytes)");
  //const UNWIND_ANY = selector("unwindDepositAny(address,bytes32,uint256)");

  const UNWIND_ANY = selector("unwindBorrow(address,bytes32,uint256,uint256,bytes,uint256)");

  // const argsClaim = encodeAbiParameters(
  //   [{ type: "uint256" }, { type: "bytes" }],
  //   [reqId, "0x"]
  // );

  const expectedArgs: Hex = asBytes(/* maybe undefined in your code */);
  const params = encodeAbiParameters(
    [{ type: "uint16" }, { type: "bytes" }, { type: "bytes" }, { type: "uint256" }],
    [5000, "0x", "0x", 0n],
  );

  const claimPrefix = encodeAbiParameters(
    [{ type: "uint256" }, { type: "bytes" }], // only reqId as prefix
    [reqId, params as Hex],
  );

  const actClaim = {
    target: pl.contractAddress as `0x${string}`,
    selector: asBytes(CLAIM_TO),
    expectedArgs: asBytes(claimPrefix),
    extraData: asBytes(),
    matchMode: 1, // PREFIX
  };

  const maxVal = BigInt(2 ** 256) - BigInt(1);

  // Action 1: CG.unwindDepositAny(user, strategy, <fill amount off-chain>)
  //   const unwindPrefix = encodeAbiParameters(
  //     [{ type: "address" }, { type: "bytes32" }, {type: "uint256"}],
  //     [alice.address, STRAT_WRAP_ID,maxVal]
  //   );

  //const connectorMinOut = await Conn.read.assetsOf([alice.address]) as bigint; //retrieve entitlement

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

    [alice.address, STRAT_WRAP_ID, maxVal, minCollateralOut, asBytes(), 1n],
  );

  //   // Action 1: CG.unwindDepositAny(user, strategy, <fill amount off-chain>)
  //   const unwindPrefix = encodeAbiParameters(
  //     [{ type: "address" }, { type: "bytes32" }, {type: "uint256"}],
  //     [alice.address, STRAT_WRAP_ID,maxVal]
  //   );

  const actUnwind = {
    target: cg.contractAddress as `0x${string}`,
    selector: asBytes(UNWIND_ANY),
    expectedArgs: asBytes(unwindPrefix),
    extraData: asBytes(),
    matchMode: 1, // PREFIX
  };
  // const argsUnwind = encodeAbiParameters(
  //   [{ type: "address" }, { type: "bytes32" }, { type: "uint256" }],
  //   [alice.address, STRAT_WRAP_ID, BigInt("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")]
  // );

  // const actions = [
  //   { target: pl.contractAddress, selector: CLAIM_TO,          expectedArgs: argsClaim,  extraData: "0x", matchMode: 1 },
  //   { target: cg.contractAddress, selector: UNWIND_ANY,        expectedArgs: argsUnwind, extraData: "0x", matchMode: 1 },
  // ];

  //   struct Action {
  //     address target;
  //     bytes4  selector;
  //     bytes   expectedArgs;     // bounded
  //     bytes32 expectedArgsHash; // keccak256(expectedArgs)
  //     bytes   extraData;        // bounded, optional
  //     MatchMode matchMode;
  // }
  const expectedArgsHash = "0x" + require("js-sha3").keccak_256(claimPrefix);
  const expectedArgsHash2 = "0x" + require("js-sha3").keccak_256(unwindPrefix);
  //const STRAT_WRAP_ID = STRAT_WRAP as `0x${string}`;
  console.log(CLAIM_TO);
  //   const actions: readonly [
  //     `0x${string}`,  // target
  //     Hex,            // selector (bytes4)
  //     Hex,            // expectedArgs (bytes)
  //     Hex,            // extraData (bytes)
  //     Hex,
  //     number          // matchMode (uint8)
  //   ][] = [
  //     [
  //       pl.contractAddress as `0x${string}`,
  //       CLAIM_TO,               // CLAIM_TO selector (bytes4)
  //       asBytes(claimPrefix),       // expectedArgs
  //       expectedArgsHash as `0x${string}`,
  //       '0x',                       // extraData
  //       2,                          // PREFIX
  //     ],
  //     [
  //       cg.contractAddress as `0x${string}`,
  //       UNWIND_ANY,               // UNWIND_ANY selector (bytes4)
  //       asBytes(unwindPrefix),      // expectedArgs
  //       expectedArgsHash2 as `0x${string}`,
  //       '0x',                       // extraData
  //       2,
  //     ],
  //   ];

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
  console.log(actions);

  console.log("openRequest");
  const txSim = await BVS.simulate.openRequest(
    [chainId, actions, completionAll, kRequired, quorumBps, minCount, ttlSeconds, operatorAllow],
    { account: owner.address },
  );

  // const txSim = await BVS.simulate.openRequest([chainId, actions, /*ALL*/ 1, /*k*/0, /*bps*/0, minCount, /*ttl*/3600, []], {
  //   account: owner.address});
  console.log("openRequest2");
  //const txSim = await bvs.contract.simulate.openRequest([chainId, actions, /*ALL*/ 1, /*k*/0, /*bps*/0, minCount, /*ttl*/3600, []], { account: owner });
  const bvsReqId = txSim.result as bigint;
  const txopen = await BVS.write.openRequest([chainId, actions, 1, 0, 0, minCount, 3600, []], { account: owner });
  await eth.mineBlock(1);
  console.log("endingRequest");
  await mineUntilReceipt(eth, eth.getClient(), txopen as any);

  // wait for node to execute both actions & attest, then finalize
  console.log("canFinalize");
  await vi.waitFor(
    async () => {
      const can = (await bvs.contract.read.canFinalize([bvsReqId])) as CanFinalizeResult;
      const ok = Boolean(can[0]);
      console.log("can", can);
      if (ok) {
        const fintx = await bvs.contract.write.finalizeRequest([bvsReqId], { account: owner });
        await mineUntilReceipt(eth, eth.getClient(), fintx as any);
      }
      expect(ok).toBe(true);
    },
    { interval: 500, timeout: 30_000 },
  );

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

  console.log("openRequest2");

  const txSimreq2 = await BVS.simulate.openRequest(
    [chainId, actions2, completionAll, kRequired, quorumBps, minCount, ttlSeconds, operatorAllow],
    { account: owner.address },
  );

  // const txSim = await BVS.simulate.openRequest([chainId, actions, /*ALL*/ 1, /*k*/0, /*bps*/0, minCount, /*ttl*/3600, []], {
  //   account: owner.address});
  console.log("openRequest2");
  //const txSim = await bvs.contract.simulate.openRequest([chainId, actions, /*ALL*/ 1, /*k*/0, /*bps*/0, minCount, /*ttl*/3600, []], { account: owner });
  const bvsReqId2 = txSimreq2.result as bigint;
  const txopen2 = await BVS.write.openRequest([chainId, actions2, 1, 0, 0, minCount, 3600, []], { account: owner });
  await eth.mineBlock(1);
  console.log("endingRequest");
  await mineUntilReceipt(eth, eth.getClient(), txopen2 as any);

  // wait for node to execute both actions & attest, then finalize
  console.log("canFinalize");
  await vi.waitFor(
    async () => {
      const can = (await bvs.contract.read.canFinalize([bvsReqId2])) as CanFinalizeResult;
      const ok = Boolean(can[0]);
      console.log("can", can);
      if (ok) {
        const fintx2 = await bvs.contract.write.finalizeRequest([bvsReqId2], { account: owner });
        await mineUntilReceipt(eth, eth.getClient(), fintx2 as any);
      }
      expect(ok).toBe(true);
    },
    { interval: 500, timeout: 30_000 },
  );

  // Assertions like your Foundry test
  // connector entitlement should be 0 after full unwind
  console.log("Readings");

  const ent = (await Conn.read.assetsOf([alice.address])) as bigint;
  console.log("Entitlement", ent);

  expect(ent).toBe(0n);

  console.log("Unlockable");
  // user should be fully unlockable for the strategy
  const unlockable = (await PL.read.unlockable([alice.address, STRAT_WRAP_ID])) as bigint;
  const totals = (await PL.read.userTotals([alice.address])) as any[];
  const totalShares2 = totals[0] as bigint;

  console.log(unlockable, totals, totalShares2, unlockable == totalShares2);
  expect(unlockable).toBe(totalShares2);

  // user can opt-out all
  const outputtx = await PL.write.optOutAll([[STRAT_WRAP_ID]], { account: alice });
  await mineUntilReceipt(eth, eth.getClient(), outputtx as any);

  // sanity: vaulted share balance returned to user
  const vBal = (await getContract({
    address: vault,
    abi: parseAbi(["function balanceOf(address) view returns (uint256)"]),
    client,
  }).read.balanceOf([alice.address])) as bigint;

  expect(vBal).toBe(totalShares2);
}, 120_000);
