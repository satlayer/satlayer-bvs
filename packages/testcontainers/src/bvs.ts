import { readFile } from "node:fs/promises";

import { InstantiateResult, SigningCosmWasmClient, UploadResult } from "@cosmjs/cosmwasm-stargate";

type BvsInitMsg = {
  "@satlayer/bvs-pauser": import("@satlayer/cosmwasm-schema/pauser").InstantiateMsg;
  "@satlayer/bvs-registry": import("@satlayer/cosmwasm-schema/registry").InstantiateMsg;
  "@satlayer/bvs-vault-router": import("@satlayer/cosmwasm-schema/vault-router").InstantiateMsg;
  "@satlayer/bvs-vault-cw20": import("@satlayer/cosmwasm-schema/vault-cw20").InstantiateMsg;
  "@satlayer/bvs-vault-bank": import("@satlayer/cosmwasm-schema/vault-bank").InstantiateMsg;
  "@satlayer/bvs-guardrail": import("@satlayer/cosmwasm-schema/guardrail").InstantiateMsg;
};

export async function uploadBvs<Pkg extends keyof BvsInitMsg>(
  client: SigningCosmWasmClient,
  sender: string,
  pkg: Pkg,
): Promise<UploadResult> {
  const path = require.resolve(`${pkg}/dist/contract.wasm`);
  const wasmCode = await readFile(path);
  return client.upload(sender, wasmCode, "auto", undefined);
}

export async function instantiateBvs<Pkg extends keyof BvsInitMsg>(
  client: SigningCosmWasmClient,
  sender: string,
  pkg: Pkg,
  codeId: number,
  initMsg: BvsInitMsg[Pkg],
  admin: string = sender,
): Promise<InstantiateResult> {
  return await client.instantiate(sender, codeId, initMsg, pkg, "auto", {
    admin: admin,
  });
}
