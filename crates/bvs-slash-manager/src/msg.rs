use crate::query::{
    CalculateSlashHashResponse, MinimalSlashSignatureResponse, SlashDetailsResponse,
    ValidatorResponse,
};
use crate::utils::ExecuteSlashDetails;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,

    pub delegation_manager: String,
    pub strategy_manager: String,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    SubmitSlashRequest {
        slash_details: ExecuteSlashDetails,
        validators_public_keys: Vec<String>,
    },
    ExecuteSlashRequest {
        slash_hash: String,
        signatures: Vec<String>,
        validators_public_keys: Vec<String>,
    },
    CancelSlashRequest {
        slash_hash: String,
    },
    SetMinimalSlashSignature {
        minimal_signature: u64,
    },
    SetSlasher {
        slasher: String,
        value: bool,
    },
    SetSlasherValidator {
        validators: Vec<String>,
        values: Vec<bool>,
    },
    SetDelegationManager {
        new_delegation_manager: String,
    },
    SetStrategyManager {
        new_strategy_manager: String,
    },
    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SlashDetailsResponse)]
    GetSlashDetails { slash_hash: String },

    #[returns(ValidatorResponse)]
    IsValidator { validator: String },

    #[returns(MinimalSlashSignatureResponse)]
    GetMinimalSlashSignature {},

    #[returns(CalculateSlashHashResponse)]
    CalculateSlashHash {
        sender: String,
        slash_details: ExecuteSlashDetails,
        validators_public_keys: Vec<String>,
    },
}
