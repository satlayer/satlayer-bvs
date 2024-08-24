use crate::query::{
    CalculateDigestHashResponse, DelegationManagerResponse, DepositTypehashResponse,
    DepositsResponse, DomainNameResponse, DomainTypehashResponse, NonceResponse, OwnerResponse,
    StakerStrategyLisResponse, StakerStrategyListLengthResponse, StakerStrategySharesResponse,
    StrategyManagerStateResponse, StrategyWhitelistedResponse, StrategyWhitelisterResponse,
    ThirdPartyTransfersForbiddenResponse,
};
use crate::utils::QueryDigestHashParams;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub delegation_manager: String,
    pub slasher: String,
    pub initial_strategy_whitelister: String,
    pub initial_owner: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddStrategiesToWhitelist {
        strategies: Vec<String>,
        third_party_transfers_forbidden_values: Vec<bool>,
    },
    RemoveStrategiesFromWhitelist {
        strategies: Vec<String>,
    },
    SetStrategyWhitelister {
        new_strategy_whitelister: String,
    },
    DepositIntoStrategy {
        strategy: String,
        token: String,
        amount: Uint128,
    },
    SetThirdPartyTransfersForbidden {
        strategy: String,
        value: bool,
    },
    DepositIntoStrategyWithSignature {
        strategy: String,
        token: String,
        amount: Uint128,
        staker: String,
        public_key: Binary,
        expiry: u64,
        signature: Binary,
    },
    RemoveShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    WithdrawSharesAsTokens {
        recipient: String,
        strategy: String,
        shares: Uint128,
        token: String,
    },
    AddShares {
        staker: String,
        token: String,
        strategy: String,
        shares: Uint128,
    },
    SetDelegationManager {
        new_delegation_manager: String,
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
    #[returns(DepositsResponse)]
    GetDeposits { staker: String },

    #[returns(StakerStrategyListLengthResponse)]
    StakerStrategyListLength { staker: String },

    #[returns(StakerStrategySharesResponse)]
    GetStakerStrategyShares { staker: String, strategy: String },

    #[returns(ThirdPartyTransfersForbiddenResponse)]
    IsThirdPartyTransfersForbidden { strategy: String },

    #[returns(NonceResponse)]
    GetNonce { staker: String },

    #[returns(StakerStrategyLisResponse)]
    GetStakerStrategyList { staker: String },

    #[returns(OwnerResponse)]
    GetOwner {},

    #[returns(StrategyWhitelistedResponse)]
    IsStrategyWhitelisted { strategy: String },

    #[returns(CalculateDigestHashResponse)]
    CalculateDigestHash {
        digst_hash_params: QueryDigestHashParams,
    },

    #[returns(StrategyWhitelisterResponse)]
    GetStrategyWhitelister {},

    #[returns(StrategyManagerStateResponse)]
    GetStrategyManagerState {},

    #[returns(DepositTypehashResponse)]
    GetDepositTypehash {},

    #[returns(DomainTypehashResponse)]
    GetDomainTypehash {},

    #[returns(DomainNameResponse)]
    GetDomainName {},

    #[returns(DelegationManagerResponse)]
    GetDelegationManager {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: Binary,
    pub salt: Binary,
    pub expiry: u64,
}
