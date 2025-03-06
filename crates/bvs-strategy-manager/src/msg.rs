use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    DepositIntoStrategy {
        strategy: String,
        token: String,
        amount: Uint128,
    },
    WithdrawSharesAsTokens {
        recipient: String,
        strategy: String,
        shares: Uint128,
    },
    AddShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    RemoveShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    TransferOwnership {
        /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
        new_owner: String,
    },
    SetRouting {
        delegation_manager: String,
        slash_manager: String,
    },
    AddStrategy {
        strategy: String,
        whitelisted: bool,
    },
    UpdateStrategy {
        strategy: String,
        whitelisted: bool,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // TODO(fuxingloh): rename this StakerDeposits { staker: String }
    #[returns(DepositsResponse)]
    GetDeposits { staker: String },

    #[returns(StakerStrategySharesResponse)]
    StakerStrategyShares { staker: String, strategy: String },

    #[returns(StakerStrategyListResponse)]
    StakerStrategyList { staker: String },

    #[returns(IsStrategyWhitelistedResponse)]
    IsStrategyWhitelisted(String),
}

#[cw_serde]
pub struct DepositsResponse {
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

#[cw_serde]
pub struct StakerStrategySharesResponse(pub Uint128);

#[cw_serde]
pub struct StakerStrategyListResponse(pub Vec<Addr>);

#[cw_serde]
pub struct IsStrategyWhitelistedResponse(pub bool);

/// Both Strategy Manager and Delegation Manager are circularly dependent on each other.
/// Since we can't circularly import each other, we put ExecuteMsg,
/// which is only used by the Strategy Manager here.
/// Delegation Manager must import this module and implement [IncreaseDelegatedShares]
pub mod delegation_manager {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::Uint128;

    #[cw_serde]
    pub enum ExecuteMsg {
        IncreaseDelegatedShares(IncreaseDelegatedShares),
    }

    #[cw_serde]
    pub struct IncreaseDelegatedShares {
        pub staker: String,
        pub strategy: String,
        pub shares: Uint128,
    }
}
