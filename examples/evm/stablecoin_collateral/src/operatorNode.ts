import { StartedAnvilContainer, SuperTestClient } from "@satlayer/testcontainers";
import {
  Account,
  getContract,
  GetContractReturnType,
  WatchContractEventReturnType,
  Hex,
  encodeFunctionData,
  PublicClient,
  AbiParameter
} from "viem";
import { anvil, mainnet } from 'viem/chains';
import { abi as bvsAbi } from "../out/StablecoinCollateralBVS.sol/StablecoinCollateralBVS.json";

import { decodeEventLog } from 'viem';
import type { Abi, Address } from 'viem';

export type BVSRequest = {
  chainId: bigint;
  completion: bigint;
  kRequired: bigint;
  quorumBps: bigint;
  minCount: bigint;
  createdAt: bigint;
  expiresAt: bigint;
  status: bigint;
  attestedCount: bigint;
  finalizedAt: bigint;
  hasOperatorAllowlist: boolean;
};

const opQueues = new Map<`0x${string}`, Promise<unknown>>();

function enqueueForOperator<T>(
  operatorAddr: `0x${string}`,
  job: () => Promise<T>
): Promise<T> {
  const prev = opQueues.get(operatorAddr) ?? Promise.resolve();
  const next = prev.then(job, job);          // chain the work
  opQueues.set(operatorAddr, next.catch(() => {})); // keep chain even on error
  return next;                                // resolves to job()'s result
}

type BVSRequestDecoded = readonly [
  bigint, // chainId
  number, // completion (enum)
  number, // kRequired (uint16 -> number)
  number, // quorumBps
  number, // minCount
  bigint, // createdAt (uint48 -> bigint)
  bigint, // expiresAt (uint48 -> bigint)
  number, // status (enum -> number)
  number, // attestedCount (uint16/uint32 -> number)
  number, // finalizedAt (uint32 -> number)
  boolean // hasOperatorAllowlist
];

export const ReqStatus = {
  Open: 0,
  Finalized: 1,
  Cancelled: 2,
  Expired: 3,
} as const;
async function mineUntilReceipt(
  eth: StartedAnvilContainer,
  client: ReturnType<StartedAnvilContainer['getClient']>,
  hash: Hex,
  maxTries = 10
) {
  for (let i = 0; i < maxTries; i++) {
    await eth.mineBlock(1);
    const r = await client.getTransactionReceipt({ hash }).catch(() => null);
    if (r) return r;
  }
  throw new Error(`Receipt not found after ${maxTries} mined blocks: ${hash}`);
}

/** BVS enums (must match solidity) */

enum MatchMode { NONE = 0, EXACT = 1, PREFIX = 2 }

/** Shape returned by a handler: either executed tx hash, or an external tx to attest, plus optional notes */
type HandlerResult = {
  txHash?: Hex;                 // tx we sent
  externallyProvidedTx?: Hex;   // tx we didn't send but will attest
  notes?: Hex;                  // arbitrary small blob to put in `extraData` on attest
};

/** What the handler receives */
export type HandlerCtx = {
  client: SuperTestClient;
  bvs: GetContractReturnType<typeof bvsAbi, SuperTestClient, `0x${string}`>;
  operator: Account;
  requestId: bigint;
  actionIndex: number;
  eth :StartedAnvilContainer;

  /** Resolved request + action */
  chainId: bigint;
  target: `0x${string}`;
  selector: Hex;          // 0x12345678
  matchMode: MatchMode;
  expectedArgs: Hex;      // may be 0x
  expectedArgsHash: Hex;
  extraData: Hex;         // optional from governance

  /** helpers */
  decodeArgs: (abi: any[]) => any[] | null;      // try to decode expectedArgs against a target ABI
  buildCalldata: (fullArgs: Hex) => Hex;         // selector + args
  canExecute: () => boolean;                     // you can implement per-operator policy (whitelists, budgets…)
};



/** Registry for target ABIs the operator knows how to call */
export interface TargetAbiRegistry {
  getAbi(target: `0x${string}`): any[] | null;
}

export class MapTargetAbiRegistry implements TargetAbiRegistry {
  private map = new Map<string, any[]>();
  add(target: `0x${string}`, abi: any[]) {
    this.map.set(target.toLowerCase(), abi);
    return this;
  }
  getAbi(target: `0x${string}`) {
    return this.map.get(target.toLowerCase()) ?? null;
  }
}

/** Default generic handler: tries to execute when possible, else no-op so we can just attest later */
// async function genericExecuteAndAttest(ctx: HandlerCtx): Promise<HandlerResult> {
//   // We need full args to execute (selector is known).
//   let argsHex: Hex | null = null;

//   argsHex = ctx.expectedArgs;

//   //TODO: implement different matchmodes


//   if (!argsHex) return {};
//   const calldata = ctx.buildCalldata(argsHex);

//   // (Policy gate)
//   if (!ctx.canExecute()) return {};

//     const nonce = await ctx.client.getTransactionCount(ctx.operator);
//     console.log("nonce", nonce)
    

//   // Actually send the tx — NOTE: in many setups, the operator will NOT be the EOA
//   // that is allowed to call the target. Plug your real sender here.
//   const txHash = await ctx.client.sendTransaction({
//     account: ctx.operator,     // Account de viem (p.ej. privateKeyToAccount)
//     to: ctx.target , // 0x...
//     data: calldata ,     // 0x...
//     chain:  mainnet,
//     nonce: nonce, 
//     });
//     await mineUntilReceipt(ctx.eth, ctx.client, txHash as any);


//     console.log("transaction mined")

//     const nonce2 = await ctx.client.getTransactionCount(ctx.operator);
//     console.log("nonce", nonce2)


//   return { txHash };
// }

/** Default generic handler: tries to execute when possible, else no-op so we can just attest later */
async function genericExecuteAndAttest(ctx: HandlerCtx): Promise<HandlerResult> {
  // Build args (prefix/full/etc). If you already have them:
  const argsHex = ctx.expectedArgs;
  if (!argsHex) return {};

  // Policy gate
  if (!ctx.canExecute()) return {};

  const calldata = ctx.buildCalldata(argsHex);

  // Queue per-operator so txs don’t race on nonce
  return enqueueForOperator(ctx.operator.address as `0x${string}`, async () => {
    const txHash = await ctx.client.sendTransaction({
      account: ctx.operator,
      to: ctx.target as `0x${string}`,
      data: calldata as `0x${string}`,
      chain: ctx.eth.getChainInfo().viemChain ,

    });

    await mineUntilReceipt(ctx.eth, ctx.client, txHash as any);

    await ctx.eth.mineBlock(20);

    return { txHash };
  });
}


/** Operator node */
export class OperatorNode {


  private unwatchOpen?: WatchContractEventReturnType;
  private unwatchAction?: WatchContractEventReturnType;

  private bvs: GetContractReturnType<typeof bvsAbi, SuperTestClient, `0x${string}`>;

  /** Handlers keyed by selector (lowercased 0x12345678) */
  private handlers = new Map<string, (ctx: HandlerCtx) => Promise<HandlerResult>>();

  constructor(
    private readonly label: string,
    private readonly client: SuperTestClient,
    private readonly container: StartedAnvilContainer,
    private readonly operator: Account,
    private readonly bvsAddress: `0x${string}`,
    private readonly targetAbiRegistry: TargetAbiRegistry,
  ) {
    this.bvs = getContract({ address: bvsAddress, abi: bvsAbi, client });


    // Default: all selectors use the generic handler unless overridden.
    // You can register specific handlers below for PL/CG functions.
    this.handlers.set("default", genericExecuteAndAttest);

    // example: register a concrete handler for CG.unwindBorrow(bytes32,address,uint256,…)
    // this.handlers.set("0x12345678", this.handleUnwindBorrow.bind(this));
  }

  /** Add or override a handler for a given 4-byte selector */
  public addHandler(selector4bytes: Hex, handler: (ctx: HandlerCtx) => Promise<HandlerResult>) {
    this.handlers.set(selector4bytes.toLowerCase(), handler);
  }

  public start() {
    console.log(`[${this.label}] Listening for RequestOpened/RequestActionAdded…`);
    // Listen for new Requests (NOTE not necessary as we only admit request with actions, leaving for info)
    // this.unwatchOpen = this.client.watchContractEvent({
    //   address: this.bvsAddress,
    //   abi: bvsAbi,
    //   eventName: "RequestOpened",
    //   onLogs: async (logs) => {
    //     console.log("getting logs");
    //     for (const log of logs) {
    //         const { args } = decodeEventLog({
    //             abi: bvsAbi,
    //             data: log.data,
    //             topics: log.topics,
    //         });

    //       const id = (args as any).id as bigint;
    //      const chainId = (args as any).chainId as bigint;

    //       console.log(`[${this.label}] New RequestOpened id=${id} on chain ${chainId}`);
    //       // Try to fetch and act now (or wait until all actions arrive; we also listen to RequestActionAdded)
    //       await this.tryProcessRequest(id);
    //     }
    //   },
    // });


    // Listen for actions being added (some setups emit actions after opening)
        this.unwatchAction = this.client.watchContractEvent({
        address: this.bvsAddress as Address,
        abi: bvsAbi,                              // <- literal (as const)
        eventName: 'RequestActionAdded' as const, // <- literal
        // strict opcional, pero ayuda a filtrar y tipar mejor
        strict: true,
        onLogs: async (logs) => {

            console.log("getting logs2");
            for (const log of logs) {
            const { args } = decodeEventLog({
                abi: bvsAbi,
                data: log.data,
                topics: log.topics,
            });

          const id = (args as any).id as bigint;
         const idx = (args as any).index as bigint;
          console.log(`[${this.label}] New RequestOpened id=${id}`);
            await this.tryProcessRequest(id);
            }
        },
        });
  }

  public stop() {
    this.unwatchOpen?.();
    this.unwatchAction?.();
  }

  /** Fetch request & actions from storage and decide what we can handle */
  private async tryProcessRequest(requestId: bigint) {
    console.log("tryProcessRequest");
    // Fetch request header
    const R = await this.bvs.read.requests([requestId as bigint]) as BVSRequestDecoded;
    const [
    chainId,
    completion,
    kRequired,
    quorumBps,
    minCount,
    createdAt,
    expiresAt,
    status,
    attestedCount,
    finalizedAt,
    hasAllow,
    ] = R;

    console.log(status)

    if (status !== ReqStatus.Open) {
    return; // no está “Open”
    }


    // Pull actions: the contract stores them in an array mapping; we don’t know length directly.
    // If your BVS exposes an actionCount(requestId) view, use it.
    // Otherwise, we can probe sequentially until it reverts.
    const actions: Array<{
      target: `0x${string}`;
      selector: Hex;
      expectedArgs: Hex;
      expectedArgsHash: Hex;
      extraData: Hex;
      matchMode: number;
    }> = [];

    let i = 0;
     console.log("reading action lengths1....")
    while (true) {
      try {
        // public mapping requestActions => Action[]; we need a view accessor. Expose one in BVS:
        // function getAction(uint256 id, uint256 idx) external view returns (Action memory)
        const A = await (this.bvs.read as any).requestActions([requestId, BigInt(i)]);
        const done = await this.bvs.read.hasAttested([requestId, BigInt(i), this.operator.address]);
        if (done) continue;
        // If your compiler generated a tuple: [target, selector, expectedArgs, expectedArgsHash, extraData, matchMode]
        actions.push({
          target: A[0],
          selector: A[1],
          expectedArgs: A[2],
          expectedArgsHash: A[3],
          extraData: A[4],
          matchMode: Number(A[5]),
        });
        i++;
      } catch {
        console.log("cannot read....")
        break;
      }
    }

    console.log("reading action lengths....")



    // Process each action individually (idempotent: we’ll skip if we already attested)
    for (let actionIndex = 0; actionIndex < actions.length; actionIndex++) {
    const already = await (this.bvs.read as any).hasAttested([
        requestId,
        BigInt(actionIndex),
        this.operator.address,
        ]);
        if (already) continue;
        console.log("action", actionIndex);
      await this.executeOrAttest(requestId, chainId as bigint, actions[actionIndex], actionIndex);
    }
  }

  private async executeOrAttest(
    requestId: bigint,
    chainId: bigint,
    A: {
      target: `0x${string}`;
      selector: Hex;
      expectedArgs: Hex;
      expectedArgsHash: Hex;
      extraData: Hex;
      matchMode: number;
    },
    actionIndex: number
  ) {
    // Build handler context
    const ctx: HandlerCtx = {
      client: this.client,
      bvs: this.bvs,
      operator: this.operator,
      eth: this.container,
      requestId,
      actionIndex,
      chainId,
      target: A.target,
      selector: A.selector,
      matchMode: A.matchMode as MatchMode,
      expectedArgs: A.expectedArgs,
      expectedArgsHash: A.expectedArgsHash,
      extraData: A.extraData,

      decodeArgs: (abi) => {
        try {
          // For display only: decode expectedArgs for UX. If this fails, we still can execute by concatenation.
          const fn = abi.find((f: any) => f.type === "function" && (f.selector?.toLowerCase?.() === A.selector.toLowerCase()
            || encodeSelector(f).toLowerCase() === A.selector.toLowerCase()));
          if (!fn) return null;
          const inputs = fn.inputs ?? [];
          // expectedArgs is raw-encoded params (no selector). We’d need the param types to decode.
          // viem doesn’t decode arbitrary param blob w/out function signature easily; skip if not trivial.
          return null;
        } catch {
          return null;
        }
      },

      buildCalldata: (fullArgs: Hex) => ((A.selector + (fullArgs as string).slice(2)) as Hex),

      canExecute: () => {
        // policy gate — put allowlists, budgets, time windows, role checks, etc.
        return true;
      },
    };



    // If we know the target ABI, we *could* decide how to populate args or run a specialized handler
    const selectorKey = (A.selector as string).toLowerCase();
    const handler = this.handlers.get(selectorKey) ?? this.handlers.get("default")!;

    // Execute or acquire an external tx hash to attest
    const res = await handler(ctx);
    const hashToAttest = (res?.txHash ?? res?.externallyProvidedTx) as Hex | undefined;

    // If we executed (or were given a hash), attest
    if (hashToAttest) {
      const attesttx = await this.bvs.write.attest([requestId, BigInt(actionIndex), hashToAttest, (res?.notes ?? "0x") as Hex], {
        account: this.operator.address,
      });

      await mineUntilReceipt(ctx.eth, ctx.client, attesttx as any);


      console.log(
        `[${this.label}] attested request=${requestId} action=${actionIndex} tx=${hashToAttest}`
      );
    } else {
      console.log(
        `[${this.label}] NOT attesting request=${requestId} action=${actionIndex} (insufficient args or policy skipped)`
      );
    }
  }
}

/** helper to compute a function selector from ABI entry (if needed) */
function encodeSelector(fn: any): Hex {
  // minimal utility: name + types → selector
  const signature = `${fn.name}(${(fn.inputs ?? []).map((i: any) => i.type).join(",")})`;
  // viem: getFunctionSelector came later; we can locally hash keccak256(signature) and take 4 bytes
  const bytes = new TextEncoder().encode(signature);
  // tiny keccak (or rely on viem's keccak256):
  const { keccak256 } = require("viem");
  const h = keccak256(bytes) as Hex;
  return ("0x" + h.slice(2, 10)) as Hex;
}
