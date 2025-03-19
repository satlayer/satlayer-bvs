// This file was automatically generated from vault-router/schema.json.
// DO NOT MODIFY IT BY HAND.

/**
 * The response to the `IsValidating` query. Not exported. This is just a wrapper around
 * `bool`, so that the schema can be generated.
 *
 * The response to the `IsWhitelisted` query. Not exported. This is just a wrapper around
 * `bool`, so that the schema can be generated.
 */
type IsValidatingResponse = boolean;

/**
 * The response to the `IsValidating` query. Not exported. This is just a wrapper around
 * `bool`, so that the schema can be generated.
 *
 * The response to the `IsWhitelisted` query. Not exported. This is just a wrapper around
 * `bool`, so that the schema can be generated.
 */
type IsWhitelistedResponse = boolean;

export interface InstantiateMsg {
  owner: string;
  pauser: string;
}

/**
 * ExecuteMsg SetVault the vault contract in the router and whitelist (true/false) it. Only
 * the `owner` can call this message.
 *
 * ExecuteMsg TransferOwnership See [`bvs_library::ownership::transfer_ownership`] for more
 * information on this field
 */
export interface ExecuteMsg {
  set_vault?: SetVault;
  transfer_ownership?: TransferOwnership;
}

export interface SetVault {
  vault: string;
  whitelisted: boolean;
}

export interface TransferOwnership {
  new_owner: string;
}

/**
 * QueryMsg IsWhitelisted: returns true if the vault is whitelisted. See
 * [`ExecuteMsg::SetVault`]
 *
 * QueryMsg IsValidating: returns true if the operator is validating services. See BVS
 * Registry for more information.
 *
 * QueryMsg ListVaults: returns a list of vaults. You can provide `limit` and `start_after`
 * to paginate the results. The max `limit` is 100.
 */
export interface QueryMsg {
  is_whitelisted?: IsWhitelisted;
  is_validating?: IsValidating;
  list_vaults?: ListVaults;
}

export interface IsValidating {
  operator: string;
}

export interface IsWhitelisted {
  vault: string;
}

export interface ListVaults {
  limit?: number | null;
  start_after?: null | string;
}

/**
 * The response to the `ListVaults` query. For pagination, the `start_after` field is the
 * last `vault` from the previous page.
 */
export interface VaultListResponse {
  vault: string;
  whitelisted: boolean;
}
