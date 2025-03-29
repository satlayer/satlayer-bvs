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
 * ExecuteMsg DeployCw20 Deploy a CW20 vault contract, the operator will be the sender of
 * this message. The `cw20` is the address of the CW20 contract.
 *
 * ExecuteMsg DeployBank Deploy a Bank vault contract, the operator will be the sender of
 * this message. The `denom` is the denomination of the native token, e.g. "ubbn" for
 * Babylon native token.
 *
 * ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
 * information on this field Only the `owner` can call this message.
 *
 * ExecuteMsg SetCodeId Set the code id for a vault type, allowing the factory to deploy
 * vaults of that type. Only the `owner` can call this message.
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
