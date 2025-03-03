use crate::query::{
    CalculateWithdrawalRootResponse, CumulativeWithdrawalsQueuedResponse,
    DelegatableSharesResponse, DelegatedResponse, OperatorDetailsResponse, OperatorResponse,
    OperatorSharesResponse, OperatorStakersResponse, StakerOptOutWindowBlocksResponse,
    WithdrawalDelayResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,

    pub min_withdrawal_delay_blocks: u64,
    pub strategies: Vec<String>,
    pub withdrawal_delay_blocks: Vec<u64>,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    RegisterAsOperator {
        operator_details: OperatorDetails,
        metadata_uri: String,
    },
    ModifyOperatorDetails {
        new_operator_details: OperatorDetails,
    },
    UpdateOperatorMetadataUri {
        metadata_uri: String,
    },
    DelegateTo {
        operator: String,
    },
    Undelegate {
        staker: String,
    },
    QueueWithdrawals {
        queued_withdrawal_params: Vec<QueuedWithdrawalParams>,
    },
    CompleteQueuedWithdrawal {
        withdrawal: Withdrawal,
        middleware_times_index: u64,
        receive_as_tokens: bool,
    },
    CompleteQueuedWithdrawals {
        withdrawals: Vec<Withdrawal>,
        middleware_times_indexes: Vec<u64>,
        receive_as_tokens: Vec<bool>,
    },
    IncreaseDelegatedShares(
        /// This is called by the strategy manager to increase the delegated shares of a staker
        /// The struct is hence owned by the strategy manager
        bvs_strategy_manager::msg::delegation_manager::IncreaseDelegatedShares,
    ),
    DecreaseDelegatedShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    SetMinWithdrawalDelayBlocks {
        new_min_withdrawal_delay_blocks: u64,
    },
    SetStrategyWithdrawalDelayBlocks {
        strategies: Vec<String>,
        withdrawal_delay_blocks: Vec<u64>,
    },
    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
    },
    SetRouting {
        strategy_manager: String,
        slash_manager: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DelegatedResponse)]
    IsDelegated { staker: String },

    #[returns(OperatorResponse)]
    IsOperator { operator: String },

    #[returns(OperatorDetailsResponse)]
    OperatorDetails { operator: String },

    #[returns(StakerOptOutWindowBlocksResponse)]
    StakerOptOutWindowBlocks { operator: String },

    #[returns(OperatorSharesResponse)]
    GetOperatorShares {
        operator: String,
        strategies: Vec<String>,
    },

    #[returns(DelegatableSharesResponse)]
    GetDelegatableShares { staker: String },

    #[returns(WithdrawalDelayResponse)]
    GetWithdrawalDelay { strategies: Vec<String> },

    #[returns(CalculateWithdrawalRootResponse)]
    CalculateWithdrawalRoot { withdrawal: Withdrawal },

    #[returns(OperatorStakersResponse)]
    GetOperatorStakers { operator: String },

    #[returns(CumulativeWithdrawalsQueuedResponse)]
    GetCumulativeWithdrawalsQueued { staker: String },
}

#[cw_serde]
pub struct OperatorDetails {
    pub staker_opt_out_window_blocks: u64,
}

#[cw_serde]
pub struct QueuedWithdrawalParams {
    pub withdrawer: Addr,
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

#[cw_serde]
pub struct Withdrawal {
    pub staker: Addr,
    pub delegated_to: Addr,
    pub withdrawer: Addr,
    pub nonce: Uint128,
    pub start_block: u64,
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}
