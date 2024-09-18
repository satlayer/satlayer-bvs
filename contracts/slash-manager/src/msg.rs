use crate::query::{MinimalSlashSignatureResponse, SlashDetailsResponse, ValidatorResponse};
use crate::utils::SlashDetails;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub delegation_manager: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitSlashRequest {
        slash_details: SlashDetails,
    },
    ExecuteSlashRequest {
        slash_hash: String,
        signatures: Vec<String>,
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
    TransferOwnership {
        new_owner: String,
    },
    Pause {},
    Unpause {},
    SetPauser {
        new_pauser: String,
    },
    SetUnpauser {
        new_unpauser: String,
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
}

#[cw_serde]
pub struct MigrateMsg {}
