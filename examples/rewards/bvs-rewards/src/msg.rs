use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner of this contract
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    DistributeRewards {
        token: String,
        amount: String,
        root: String,
    },

    ClaimRewards {
        token: String,
        amount: String,
        proof: Vec<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // TODO: Add query messages to retrieve root
}

#[cfg(test)]
mod tests {}
