import { readFile } from "node:fs/promises";
import { join } from "node:path";

import { InstantiateResult, SigningCosmWasmClient, UploadResult } from "@cosmjs/cosmwasm-stargate";

export type Cw20InitMsg = {
  decimals: number;
  name: string;
  symbol: string;
  initial_balances: {
    address: string;
    amount: string;
  }[];
};

export async function uploadCw20(client: SigningCosmWasmClient, sender: string): Promise<UploadResult> {
  const wasmCode = await readFile(join(__dirname, "../cw20.wasm"));
  return await client.upload(sender, wasmCode, "auto", undefined);
}

export async function instantiateCw20(
  client: SigningCosmWasmClient,
  sender: string,
  codeId: number,
  initMsg: Cw20InitMsg,
  admin: string = sender,
): Promise<InstantiateResult> {
  return await client.instantiate(sender, codeId, initMsg, initMsg.symbol, "auto", {
    admin: admin,
  });
}
