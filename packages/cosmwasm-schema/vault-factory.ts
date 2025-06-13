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
 * ExecuteMsg DeployCw20Tokenized Deploy a Cw20 tokenized vault contract, the operator will
 * be the sender of this message. The `symbol` is the symbol for the receipt token. Must
 * start with sat and conform the Bank symbol rules. The `name` is the cw20 compliant name
 * for the receipt token.
 *
 * ExecuteMsg DeployBank Deploy a Bank vault contract, the operator will be the sender of
 * this message. The `denom` is the denomination of the native token, e.g. "ubbn" for
 * Babylon native token.
 *
 * ExecuteMsg DeployBankTokenized Deploy a Bank tokenized vault contract, the operator will
 * be the sender of this message. The `denom` is the denomination of the native token, e.g.
 * "ubbn" for Babylon native token. The `decimals` is the number of decimals for the receipt
 * token The `symbol` is the symbol for the receipt token. Must start with sat and conform
 * the Bank symbol rules. The `name` is the cw20 compliant name for the receipt token.
 *
 * ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
 * information on this field Only the `owner` can call this message.
 *
 * ExecuteMsg SetCodeId Set the code id for a vault type, allowing the factory to deploy
 * vaults of that type. Only the `owner` can call this message.
 *
 * ExecuteMsg MigrateVault Migrate an existing vault to a new code id. The `vault` is the
 * address of the vault to migrate. The `vault_type` is the type of the vault to migrate.
 * Note that this execute message assumes setCodeId message has been called prior with new
 * code id for the vault type.
 */
export interface ExecuteMsg {
  deploy_cw20?: DeployCw20;
  deploy_cw20_tokenized?: DeployCw20Tokenized;
  deploy_bank?: DeployBank;
  deploy_bank_tokenized?: DeployBankTokenized;
  transfer_ownership?: TransferOwnership;
  set_code_id?: SetCodeID;
  migrate_vault?: MigrateVault;
}

export interface DeployBank {
  denom: string;
}

export interface DeployBankTokenized {
  decimals: number;
  denom: string;
  name: string;
  symbol: string;
}

export interface DeployCw20 {
  cw20: string;
}

export interface DeployCw20Tokenized {
  cw20: string;
  name: string;
  symbol: string;
}

export interface MigrateVault {
  migrate_msg: string;
  vault_address: string;
  vault_type: VaultType;
}

export enum VaultType {
  Bank = "bank",
  BankTokenized = "bank_tokenized",
  Cw20 = "cw20",
  Cw20Tokenized = "cw20_tokenized",
}

export interface SetCodeID {
  code_id: number;
  vault_type: VaultType;
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
