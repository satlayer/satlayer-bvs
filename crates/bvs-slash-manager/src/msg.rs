use crate::query::{
    CalculateSlashHashResponse, MinimalSlashSignatureResponse, SlashDetailsResponse,
    ValidatorResponse,
};
use crate::utils::ExecuteSlashDetails;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub delegation_manager: String,
    pub strategy_manager: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u8,
}

#[cw_serde]
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
    /// Transfer ownership of the contract to a new owner.
    /// Contract admin (set for all BVS contracts, a cosmwasm feature)
    /// has the omni-ability to override by migration;
    /// this logic is app-level.
    /// > 2-step ownership transfer is mostly redundant for CosmWasm contracts with the admin set.
    /// > You can override ownership with using CosmWasm migrate `entry_point`.
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

    #[returns(CalculateSlashHashResponse)]
    CalculateSlashHash {
        sender: String,
        slash_details: ExecuteSlashDetails,
        validators_public_keys: Vec<String>,
    },
}
