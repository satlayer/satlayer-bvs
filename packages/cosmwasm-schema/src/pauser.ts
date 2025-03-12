import * as z from "zod";

export const InstantiateMsgSchema = z.object({
  initial_paused: z.boolean(),
  owner: z.string(),
});
export type InstantiateMsg = z.infer<typeof InstantiateMsgSchema>;

export const PauseSchema = z.object({});
export type Pause = z.infer<typeof PauseSchema>;

export const TransferOwnershipSchema = z.object({
  new_owner: z.string(),
});
export type TransferOwnership = z.infer<typeof TransferOwnershipSchema>;

export const UnpauseSchema = z.object({});
export type Unpause = z.infer<typeof UnpauseSchema>;

export const CanExecuteSchema = z.object({
  c: z.string(),
  m: z.string(),
  s: z.string(),
});
export type CanExecute = z.infer<typeof CanExecuteSchema>;

export const IsPausedSchema = z.object({
  c: z.string(),
  m: z.string(),
});
export type IsPaused = z.infer<typeof IsPausedSchema>;

export const ExecuteMsgSchema = z.object({
  pause: PauseSchema.optional(),
  unpause: UnpauseSchema.optional(),
  transfer_ownership: TransferOwnershipSchema.optional(),
});
export type ExecuteMsg = z.infer<typeof ExecuteMsgSchema>;

export const QueryMsgSchema = z.object({
  is_paused: IsPausedSchema.optional(),
  can_execute: CanExecuteSchema.optional(),
});
export type QueryMsg = z.infer<typeof QueryMsgSchema>;
