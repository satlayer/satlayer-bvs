use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub aggregator: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateNewTask {
        input: i64,
    },
    RespondToTask {
        task_id: u64,
        result: i64,
        operators: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(i64)]
    GetTaskInput { task_id: u64 },

    #[returns(i64)]
    GetTaskResult { task_id: u64 },

    #[returns(u64)]
    GetLatestTaskId {},
}
