import * as z from "zod";

export const InstantiateMsgSchema = z.object({
  owner: z.string(),
  pauser: z.string(),
});
export type InstantiateMsg = z.infer<typeof InstantiateMsgSchema>;

export const DeregisterOperatorFromServiceSchema = z.object({
  operator: z.string(),
});
export type DeregisterOperatorFromService = z.infer<typeof DeregisterOperatorFromServiceSchema>;

export const DeregisterServiceFromOperatorSchema = z.object({
  service: z.string(),
});
export type DeregisterServiceFromOperator = z.infer<typeof DeregisterServiceFromOperatorSchema>;

export const MetadataSchema = z.object({
  name: z.union([z.null(), z.string()]).optional(),
  uri: z.union([z.null(), z.string()]).optional(),
});
export type Metadata = z.infer<typeof MetadataSchema>;

export const RegisterAsServiceSchema = z.object({
  metadata: MetadataSchema,
});
export type RegisterAsService = z.infer<typeof RegisterAsServiceSchema>;

export const RegisterOperatorToServiceSchema = z.object({
  operator: z.string(),
});
export type RegisterOperatorToService = z.infer<typeof RegisterOperatorToServiceSchema>;

export const RegisterServiceToOperatorSchema = z.object({
  service: z.string(),
});
export type RegisterServiceToOperator = z.infer<typeof RegisterServiceToOperatorSchema>;

export const TransferOwnershipSchema = z.object({
  new_owner: z.string(),
});
export type TransferOwnership = z.infer<typeof TransferOwnershipSchema>;

export const StatusSchema = z.object({
  operator: z.string(),
  service: z.string(),
});
export type Status = z.infer<typeof StatusSchema>;

export const RegisterAsOperatorSchema = z.object({
  metadata: MetadataSchema,
});
export type RegisterAsOperator = z.infer<typeof RegisterAsOperatorSchema>;

export const QueryMsgSchema = z.object({
  status: StatusSchema.optional(),
  is_service: z.string().optional(),
  is_operator: z.string().optional(),
});
export type QueryMsg = z.infer<typeof QueryMsgSchema>;

export const ExecuteMsgSchema = z.object({
  register_as_service: RegisterAsServiceSchema.optional(),
  update_service_metadata: MetadataSchema.optional(),
  register_as_operator: RegisterAsOperatorSchema.optional(),
  update_operator_metadata: MetadataSchema.optional(),
  register_operator_to_service: RegisterOperatorToServiceSchema.optional(),
  deregister_operator_from_service: DeregisterOperatorFromServiceSchema.optional(),
  register_service_to_operator: RegisterServiceToOperatorSchema.optional(),
  deregister_service_from_operator: DeregisterServiceFromOperatorSchema.optional(),
  transfer_ownership: TransferOwnershipSchema.optional(),
});
export type ExecuteMsg = z.infer<typeof ExecuteMsgSchema>;
