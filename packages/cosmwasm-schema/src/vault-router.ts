import * as z from "zod";

export const InstantiateMsgSchema = z.object({
  owner: z.string(),
  pauser: z.string(),
});
export type InstantiateMsg = z.infer<typeof InstantiateMsgSchema>;

export const SetVaultSchema = z.object({
  vault: z.string(),
  whitelisted: z.boolean(),
});
export type SetVault = z.infer<typeof SetVaultSchema>;

export const TransferOwnershipSchema = z.object({
  new_owner: z.string(),
});
export type TransferOwnership = z.infer<typeof TransferOwnershipSchema>;

export const IsValidatingSchema = z.object({
  operator: z.string(),
});
export type IsValidating = z.infer<typeof IsValidatingSchema>;

export const IsWhitelistedSchema = z.object({
  vault: z.string(),
});
export type IsWhitelisted = z.infer<typeof IsWhitelistedSchema>;

export const ListVaultsSchema = z.object({
  limit: z.union([z.number(), z.null()]).optional(),
  start_after: z.union([z.null(), z.string()]).optional(),
});
export type ListVaults = z.infer<typeof ListVaultsSchema>;

export const VaultSchema = z.object({
  vault: z.string(),
  whitelisted: z.boolean(),
});
export type Vault = z.infer<typeof VaultSchema>;

export const ExecuteMsgSchema = z.object({
  set_vault: SetVaultSchema.optional(),
  transfer_ownership: TransferOwnershipSchema.optional(),
});
export type ExecuteMsg = z.infer<typeof ExecuteMsgSchema>;

export const QueryMsgSchema = z.object({
  is_whitelisted: IsWhitelistedSchema.optional(),
  is_validating: IsValidatingSchema.optional(),
  list_vaults: ListVaultsSchema.optional(),
});
export type QueryMsg = z.infer<typeof QueryMsgSchema>;
