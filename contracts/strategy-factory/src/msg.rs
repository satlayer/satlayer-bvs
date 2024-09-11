use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_owner: String,
    pub strategy_code_id: u64,
    pub only_owner_can_create: bool,
    pub strategy_manager: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    // 创建新的策略，传入token地址以及可选的 pauser 和 unpauser 地址
    CreateStrategy {
        token: String,
        pauser: String,   // 可选的 pauser 地址
        unpauser: String, // 可选的 unpauser 地址
    },

    // 更新合约配置
    UpdateConfig {
        new_owner: String,           // 可选的新 owner
        strategy_code_id: u64,       // 可选的新策略 code ID
        only_owner_can_create: bool, // 是否只有 owner 可以创建策略
    },
}

#[cw_serde]
pub enum QueryMsg {
    // 获取策略的接口，通过 token_address 查询
    GetStrategy { token: String },
}
