use bvs_pauser::api::Display;
use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::VaultType;

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
        cw20: String,
    },

    DeployBank {
        denom: String,
    },

    /// ExecuteMsg TransferOwnership
    /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
    /// Only the `owner` can call this message.
    TransferOwnership {
        new_owner: String,
    },

    SetCodeId {
        code_id: u64,
        label: VaultType,
    },

    RemoveCodeId {
        label: VaultType,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VaultCodeIdsResponse)]
    GetVaultCodeIds {},
}

#[cw_serde]
pub struct VaultCodeIdsResponse {
    pub code_ids: Vec<(VaultType, u64)>,
}
