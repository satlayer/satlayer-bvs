use crate::query::{
    CalculateWithdrawalRootResponse, CumulativeWithdrawalsQueuedResponse,
    CurrentStakerDelegationDigestHashResponse, DelegatableSharesResponse, DelegatedResponse,
    DelegationApprovalDigestHashResponse, DelegationApproverResponse, OperatorDetailsResponse,
    OperatorResponse, OperatorSharesResponse, OperatorStakersResponse,
    StakerDelegationDigestHashResponse, StakerNonceResponse, StakerOptOutWindowBlocksResponse,
    WithdrawalDelayResponse,
};
use crate::utils::{
    ExecuteDelegateParams, QueryApproverDigestHashParams, QueryCurrentStakerDigestHashParams,
    QueryStakerDigestHashParams, Withdrawal,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub strategy_manager: String,
    pub slash_manager: String,
    pub min_withdrawal_delay_blocks: u64,
    pub initial_owner: String,
    pub strategies: Vec<String>,
    pub withdrawal_delay_blocks: Vec<u64>,
    pub registry: String,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    RegisterAsOperator {
        sender_public_key: String,
        operator_details: ExecuteOperatorDetails,
        metadata_uri: String,
    },
    ModifyOperatorDetails {
        new_operator_details: ExecuteOperatorDetails,
    },
    UpdateOperatorMetadataUri {
        metadata_uri: String,
    },
    DelegateTo {
        params: ExecuteDelegateParams,
        approver_signature_and_expiry: ExecuteSignatureWithExpiry,
    },
    DelegateToBySignature {
        params: ExecuteDelegateParams,
        staker_public_key: String,
        staker_signature_and_expiry: ExecuteSignatureWithExpiry,
        approver_signature_and_expiry: ExecuteSignatureWithExpiry,
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

    #[returns(DelegationApproverResponse)]
    DelegationApprover { operator: String },

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

    #[returns(StakerDelegationDigestHashResponse)]
    StakerDelegationDigestHash {
        staker_digest_hash_params: QueryStakerDigestHashParams,
    },

    #[returns(DelegationApprovalDigestHashResponse)]
    DelegationApprovalDigestHash {
        approver_digest_hash_params: QueryApproverDigestHashParams,
    },

    #[returns(CurrentStakerDelegationDigestHashResponse)]
    CalculateCurrentStakerDelegationDigestHash {
        current_staker_digest_hash_params: QueryCurrentStakerDigestHashParams,
    },

    #[returns(StakerNonceResponse)]
    GetStakerNonce { staker: String },

    #[returns(OperatorStakersResponse)]
    GetOperatorStakers { operator: String },

    #[returns(CumulativeWithdrawalsQueuedResponse)]
    GetCumulativeWithdrawalsQueued { staker: String },
}

#[cw_serde]
pub struct OperatorDetails {
    pub deprecated_earnings_receiver: Addr,
    pub delegation_approver: Addr,
    pub staker_opt_out_window_blocks: u64,
}

#[cw_serde]
pub struct ExecuteOperatorDetails {
    pub deprecated_earnings_receiver: String,
    pub delegation_approver: String,
    pub staker_opt_out_window_blocks: u64,
}

#[cw_serde]
pub struct QueuedWithdrawalParams {
    pub withdrawer: Addr,
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

#[cw_serde]
pub struct SignatureWithExpiry {
    pub signature: Binary,
    pub expiry: u64,
}

#[cw_serde]
pub struct ExecuteSignatureWithExpiry {
    pub signature: String,
    pub expiry: u64,
}
