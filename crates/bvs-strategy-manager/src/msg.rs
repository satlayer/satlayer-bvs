use crate::query::{
    CalculateDigestHashResponse, DelegationManagerResponse, DepositTypeHashResponse,
    DepositsResponse, DomainNameResponse, DomainTypeHashResponse, IsTokenBlacklisted,
    NonceResponse, OwnerResponse, StakerStrategyListLengthResponse, StakerStrategyListResponse,
    StakerStrategySharesResponse, StrategyManagerStateResponse, StrategyWhitelistedResponse,
    StrategyWhitelisterResponse, ThirdPartyTransfersForbiddenResponse, TokenStrategyResponse,
};
use crate::utils::QueryDigestHashParams;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub delegation_manager: String,
    pub slash_manager: String,
    pub strategy_factory: String,
    pub initial_strategy_whitelister: String,
    pub initial_owner: String,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u8,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddNewStrategy {
        new_strategy: String,
        token: String,
    },
    BlacklistTokens {
        tokens: Vec<String>,
    },
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
        public_key: String,
        expiry: u64,
        signature: String,
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
    SetSlashManager {
        new_slash_manager: String,
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

    #[returns(StakerStrategyListResponse)]
    GetStakerStrategyList { staker: String },

    #[returns(OwnerResponse)]
    Owner {},

    #[returns(StrategyWhitelistedResponse)]
    IsStrategyWhitelisted { strategy: String },

    #[returns(CalculateDigestHashResponse)]
    CalculateDigestHash {
        digest_hash_params: QueryDigestHashParams,
    },

    #[returns(StrategyWhitelisterResponse)]
    GetStrategyWhitelister {},

    #[returns(StrategyManagerStateResponse)]
    GetStrategyManagerState {},

    #[returns(DepositTypeHashResponse)]
    GetDepositTypeHash {},

    #[returns(DomainTypeHashResponse)]
    DomainTypeHash {},

    #[returns(DomainNameResponse)]
    DomainName {},

    #[returns(DelegationManagerResponse)]
    DelegationManager {},

    #[returns(IsTokenBlacklisted)]
    IsTokenBlacklisted { token: String },

    #[returns(TokenStrategyResponse)]
    GetTokenStrategy { token: String },
}
