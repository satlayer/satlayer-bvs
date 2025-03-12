import * as z from "zod";

export const InstantiateMsgSchema = z.object({
  denom: z.string(),
  operator: z.string(),
  pauser: z.string(),
  router: z.string(),
});
export type InstantiateMsg = z.infer<typeof InstantiateMsgSchema>;

export const RecipientAmountSchema = z.object({
  amount: z.string(),
  recipient: z.string(),
});
export type RecipientAmount = z.infer<typeof RecipientAmountSchema>;

export const AssetsSchema = z.object({
  staker: z.string(),
});
export type Assets = z.infer<typeof AssetsSchema>;

export const ConvertToAssetsSchema = z.object({
  shares: z.string(),
});
export type ConvertToAssets = z.infer<typeof ConvertToAssetsSchema>;

export const ConvertToSharesSchema = z.object({
  assets: z.string(),
});
export type ConvertToShares = z.infer<typeof ConvertToSharesSchema>;

export const SharesSchema = z.object({
  staker: z.string(),
});
export type Shares = z.infer<typeof SharesSchema>;

export const TotalAssetsSchema = z.object({});
export type TotalAssets = z.infer<typeof TotalAssetsSchema>;

export const TotalSharesSchema = z.object({});
export type TotalShares = z.infer<typeof TotalSharesSchema>;

export const VaultInfoSchema = z.object({});
export type VaultInfo = z.infer<typeof VaultInfoSchema>;

export const VaultInfoResponseSchema = z.object({
  asset_id: z.string(),
  contract: z.string(),
  operator: z.string(),
  pauser: z.string(),
  router: z.string(),
  slashing: z.boolean(),
  total_assets: z.string(),
  total_shares: z.string(),
  version: z.string(),
});
export type VaultInfoResponse = z.infer<typeof VaultInfoResponseSchema>;

export const ExecuteMsgSchema = z.object({
  deposit: RecipientAmountSchema.optional(),
  withdraw: RecipientAmountSchema.optional(),
});
export type ExecuteMsg = z.infer<typeof ExecuteMsgSchema>;

export const QueryMsgSchema = z.object({
  shares: SharesSchema.optional(),
  assets: AssetsSchema.optional(),
  convert_to_assets: ConvertToAssetsSchema.optional(),
  convert_to_shares: ConvertToSharesSchema.optional(),
  total_shares: TotalSharesSchema.optional(),
  total_assets: TotalAssetsSchema.optional(),
  vault_info: VaultInfoSchema.optional(),
});
export type QueryMsg = z.infer<typeof QueryMsgSchema>;
