import * as z from "zod";

export const InstantiateMsgSchema = z.object({
  owner: z.string(),
  pauser: z.string(),
});
export type InstantiateMsg = z.infer<typeof InstantiateMsgSchema>;

export const OperatorDeregisterServiceSchema = z.object({
  service: z.string(),
});
export type OperatorDeregisterService = z.infer<typeof OperatorDeregisterServiceSchema>;

export const MetadataSchema = z.object({
  name: z.union([z.null(), z.string()]).optional(),
  uri: z.union([z.null(), z.string()]).optional(),
});
export type Metadata = z.infer<typeof MetadataSchema>;

export const OperatorRegisterServiceSchema = z.object({
  service: z.string(),
});
export type OperatorRegisterService = z.infer<typeof OperatorRegisterServiceSchema>;

export const ServiceDeregisterOperatorSchema = z.object({
  operator: z.string(),
});
export type ServiceDeregisterOperator = z.infer<typeof ServiceDeregisterOperatorSchema>;

export const ServiceRegisterSchema = z.object({
  metadata: MetadataSchema,
});
export type ServiceRegister = z.infer<typeof ServiceRegisterSchema>;

export const ServiceRegisterOperatorSchema = z.object({
  operator: z.string(),
});
export type ServiceRegisterOperator = z.infer<typeof ServiceRegisterOperatorSchema>;

export const TransferOwnershipSchema = z.object({
  new_owner: z.string(),
});
export type TransferOwnership = z.infer<typeof TransferOwnershipSchema>;

export const RegistrationStatusSchema = z.object({
  operator: z.string(),
  service: z.string(),
});
export type RegistrationStatus = z.infer<typeof RegistrationStatusSchema>;

export const OperatorRegisterSchema = z.object({
  metadata: MetadataSchema,
});
export type OperatorRegister = z.infer<typeof OperatorRegisterSchema>;

export const QueryMsgSchema = z.object({
  registration_status: RegistrationStatusSchema,
});
export type QueryMsg = z.infer<typeof QueryMsgSchema>;

export const ExecuteMsgSchema = z.object({
  service_register: ServiceRegisterSchema.optional(),
  service_update_metadata: MetadataSchema.optional(),
  service_register_operator: ServiceRegisterOperatorSchema.optional(),
  service_deregister_operator: ServiceDeregisterOperatorSchema.optional(),
  operator_register: OperatorRegisterSchema.optional(),
  operator_update_metadata: MetadataSchema.optional(),
  operator_deregister_service: OperatorDeregisterServiceSchema.optional(),
  operator_register_service: OperatorRegisterServiceSchema.optional(),
  transfer_ownership: TransferOwnershipSchema.optional(),
});
export type ExecuteMsg = z.infer<typeof ExecuteMsgSchema>;
