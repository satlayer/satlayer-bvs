use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Uint64, Binary};
use crate::utils::DigestHashParams;
use crate::state::StrategyManagerState;

#[cw_serde]
pub struct InstantiateMsg {
    pub delegation_manager: Addr,
    pub slasher: Addr,
    pub initial_strategy_whitelister: Addr,
    pub initial_owner: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddStrategiesToWhitelist {
        strategies: Vec<Addr>,
        third_party_transfers_forbidden_values: Vec<bool>,
    },
    RemoveStrategiesFromWhitelist {
        strategies: Vec<Addr>,
    },
    SetStrategyWhitelister {
        new_strategy_whitelister: Addr,
    },
    DepositIntoStrategy {
        strategy: Addr,
        token: Addr,
        amount: Uint128,
    },
    SetThirdPartyTransfersForbidden {
        strategy: Addr,
        value: bool,
    },
    DepositIntoStrategyWithSignature {
        strategy: Addr,
        token: Addr,
        amount: Uint128,
        staker: Addr,
        public_key: Binary,
        expiry: Uint64,
        signature: Binary,
    },
    RemoveShares {
        staker: Addr,
        strategy: Addr,
        shares: Uint128,
    },
    WithdrawSharesAsTokens {
        recipient: Addr,
        strategy: Addr,
        shares: Uint128,
        token: Addr,
    },
    AddShares {
        staker: Addr,
        token: Addr,
        strategy: Addr,
        shares: Uint128,
    }, 
    SetDelegationManager { new_delegation_manager: Addr },
    TransferOwnership {
        new_owner: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DepositsResponse)]
    GetDeposits { staker: Addr },

    #[returns(Uint64)]
    StakerStrategyListLength { staker: Addr },

    #[returns(Uint128)]
    GetStakerStrategyShares { staker: Addr, strategy: Addr },

    #[returns(bool)]
    IsThirdPartyTransfersForbidden { strategy: Addr },

    #[returns(Uint64)]
    GetNonce { staker: Addr },
    
    #[returns(Vec<Addr>)]
    GetStakerStrategyList { staker: Addr },

    #[returns(Addr)]
    GetOwner {},

    #[returns(bool)]
    IsStrategyWhitelisted { strategy: Addr },

    #[returns(Binary)]
    CalculateDigestHash { digst_hash_params: DigestHashParams },

    #[returns(Addr)]
    GetStrategyWhitelister {},

    #[returns(StrategyManagerState)]
    GetStrategyManagerState {},

    #[returns(String)]
    GetDepositTypehash,

    #[returns(String)]
    GetDomainTypehash,

    #[returns(String)]
    GetDomainName,

    #[returns(Addr)]
    GetDelegationManager {},
}

#[cw_serde]
pub struct SignatureWithSaltAndExpiry {
    pub signature: Binary,
    pub salt: Binary,
    pub expiry: Uint64,
}

#[cw_serde]
pub struct DepositsResponse {
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}