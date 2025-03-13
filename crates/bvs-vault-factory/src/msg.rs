use bvs_pauser::api::Display;
use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::CodeIdLabel;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub pauser: String,
    pub registry: String,
    pub router: String,
}

#[cw_serde]
#[derive(Display)]
pub enum ExecuteMsg {
    DeployCw20 {
        code_id: u64,
        cw20: String,
    },

    DeployBank {
        code_id: u64,
        denom: String,
    },

    /// ExecuteMsg TransferOwnership
    /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
    /// Only the `owner` can call this message.
    TransferOwnership {
        new_owner: String,
    },

    SetVaults {
        router: String,
        registry: String,
    },

    AddCodeId {
        code_id: u64,
        label: CodeIdLabel,
    },

    RemoveCodeId {
        code_id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AllowedCodeIdsResponse)]
    GetAllowedCodeIds {},
}

#[cw_serde]
pub struct AllowedCodeIdsResponse {
    pub code_ids: Vec<(u64, CodeIdLabel)>,
}
