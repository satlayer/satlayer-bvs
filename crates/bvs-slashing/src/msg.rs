use cosmwasm_schema::cw_serde;

/// Instantiate message
/// router: address of the router contract
/// registry: address of the registry contract
/// pauser: address of the pauser contract
/// owner: address of the owner of the contract
/// slasher: <- middleware identity ?
/// vault: address of the underlying vault contract, Currently cw20 or bank vault.
pub struct InstantiateMsg {
    pub vault: String,
    pub router: String,
    pub registry: String,
    pub pauser: String,
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SlashRequest {
        vault: String,
        share_amount: u64,
        accused: String,
        start: u64,
        end: u64,
    },
    SlashVote {
        // to be implemented
        // slash_hash: string,
        // vote: bool,
        // voter: String,
    },
    SlashExecute {
        // to be implemented
        // slash_hash: string,
    },
    AddSlasher {
        slasher: String,
    },
}
