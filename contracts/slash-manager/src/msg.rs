use cosmwasm_schema::cw_serde;

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
    RegisterAVS {
        avs_contract: String,
        state_bank: String,
        avs_driver: String,
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
    #[returns(OperatorStatusResponse)]
    GetOperatorStatus { avs: String, operator: String },
}

#[cw_serde]
pub struct MigrateMsg {}