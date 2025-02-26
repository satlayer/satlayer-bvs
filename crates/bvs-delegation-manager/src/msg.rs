use crate::query::{
    CalculateWithdrawalRootResponse, CumulativeWithdrawalsQueuedResponse,
    DelegatableSharesResponse, DelegatedResponse, OperatorDetailsResponse, OperatorResponse,
    OperatorSharesResponse, OperatorStakersResponse, StakerOptOutWindowBlocksResponse,
    WithdrawalDelayResponse,
};
use crate::utils::{ExecuteDelegateParams, Withdrawal};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub strategy_manager: String,
    pub slash_manager: String,
    pub min_withdrawal_delay_blocks: u64,
    pub initial_owner: String,
    pub strategies: Vec<String>,
    pub withdrawal_delay_blocks: Vec<u64>,
    pub pauser: String,
    pub unpauser: String,
    pub initial_paused_status: u8,
}

#[cw_serde]
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
        params: ExecuteDelegateParams,
    },
    Undelegate {
        staker: String,
    },
    QueueWithdrawals {
        queued_withdrawal_params: Vec<QueuedWithdrawalParams>,
    },
    CompleteQueuedWithdrawal {
        withdrawal: Withdrawal,
        tokens: Vec<Addr>,
        middleware_times_index: u64,
        receive_as_tokens: bool,
    },
    CompleteQueuedWithdrawals {
        withdrawals: Vec<Withdrawal>,
        tokens: Vec<Vec<Addr>>,
        middleware_times_indexes: Vec<u64>,
        receive_as_tokens: Vec<bool>,
    },
    IncreaseDelegatedShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    DecreaseDelegatedShares {
        staker: String,
        strategy: String,
        shares: Uint128,
    },
    SetMinWithdrawalDelayBlocks {
        new_min_withdrawal_delay_blocks: u64,
    },
    SetSlashManager {
        new_slash_manager: String,
    },
    SetStrategyWithdrawalDelayBlocks {
        strategies: Vec<String>,
        withdrawal_delay_blocks: Vec<u64>,
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
