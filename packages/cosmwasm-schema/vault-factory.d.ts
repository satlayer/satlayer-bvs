// This file was automatically generated from vault-factory/schema.json.
// DO NOT MODIFY IT BY HAND.

/**
 * The response to the `CodeId` query. Not exported. This is just a wrapper around `u64`, so
 * that the schema can be generated.
 */
type CodeIDResponse = number;

export interface InstantiateMsg {
  owner: string;
  pauser: string;
  registry: string;
  router: string;
}

/**
 * ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
 * information on this field Only the `owner` can call this message.
 */
export interface ExecuteMsg {
  deploy_cw20?: DeployCw20;
  deploy_bank?: DeployBank;
  transfer_ownership?: TransferOwnership;
  set_code_id?: SetCodeID;
}

export interface DeployBank {
  denom: string;
}

export interface DeployCw20 {
  cw20: string;
}

export interface SetCodeID {
  code_id: number;
  vault_type: VaultType;
}

export enum VaultType {
  Bank = "bank",
  Cw20 = "cw20",
}

export interface TransferOwnership {
  new_owner: string;
}

export interface QueryMsg {
  code_id: CodeID;
}

export interface CodeID {
  vault_type: VaultType;
}
