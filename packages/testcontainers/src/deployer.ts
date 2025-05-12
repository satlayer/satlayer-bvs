import { readFile } from "node:fs/promises";
import { join } from "node:path";

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

interface InstantiateMsg {
  "@satlayer/bvs-pauser": import("@satlayer/cosmwasm-schema/pauser").InstantiateMsg;
  "@satlayer/bvs-registry": import("@satlayer/cosmwasm-schema/registry").InstantiateMsg;
  "@satlayer/bvs-vault-router": import("@satlayer/cosmwasm-schema/vault-router").InstantiateMsg;
  "@satlayer/bvs-vault-cw20": import("@satlayer/cosmwasm-schema/vault-cw20").InstantiateMsg;
  "@satlayer/bvs-vault-bank": import("@satlayer/cosmwasm-schema/vault-bank").InstantiateMsg;
}

export async function deploy<Pkg extends keyof InstantiateMsg>(
  client: SigningCosmWasmClient,
  address: string,
  pkg: Pkg,
  initMsg: InstantiateMsg[Pkg],
) {
  const path = require.resolve(`${pkg}/dist/contract.wasm`);
  const bytecode = await readFile(path);

  const uploaded = await client.upload(address, bytecode, "auto", undefined);

  const instantiated = await client.instantiate(address, uploaded.codeId, initMsg, pkg, "auto", {
    admin: address,
  });

  return {
    codeId: uploaded.codeId,
    address: instantiated.contractAddress,
  };
}

export async function deployCw20(
  client: SigningCosmWasmClient,
  address: string,
  initMsg: {
    decimals: number;
    name: string;
    symbol: string;
    initial_balances: {
      address: string;
      amount: string;
    }[];
  },
) {
  const bytecode = await readFile(join(__dirname, "../cw20.wasm"));
  const uploaded = await client.upload(address, bytecode, "auto", undefined);

  const instantiated = await client.instantiate(address, uploaded.codeId, initMsg, initMsg.name, "auto", {
    admin: address,
  });

  return {
    codeId: uploaded.codeId,
    address: instantiated.contractAddress,
  };
}
