use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};
use crate::utils::{Withdrawal, CurrentStakerDigestHashParams, StakerDigestHashParams, ApproverDigestHashParams};

#[cw_serde]
pub struct InstantiateMsg {
    pub strategy_manager: Addr,
    pub slasher: Addr,
    pub min_withdrawal_delay_blocks: u64,
    pub initial_owner: Addr,
    pub strategies: Vec<Addr>,
    pub withdrawal_delay_blocks: Vec<u64>,
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
        staker: Addr,
        approver_signature_and_expiry: SignatureWithExpiry,
        approver_salt: Binary,
    },
    DelegateToBySignature {
        staker: Addr,
        operator: Addr,
        staker_signature_and_expiry: SignatureWithExpiry,
        approver_signature_and_expiry: SignatureWithExpiry,
        approver_salt: Binary,
    },
    Undelegate {
        staker: Addr,
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
        staker: Addr,
        strategy: Addr,
        shares: u128,
    },
    DecreaseDelegatedShares {
        staker: Addr,
        strategy: Addr,
        shares: u128,
    },
    SetMinWithdrawalDelayBlocks {
        new_min_withdrawal_delay_blocks: u64,
    },
    SetStrategyWithdrawalDelayBlocks {
        strategies: Vec<Addr>,
        withdrawal_delay_blocks: Vec<u64>,
    },
    IncreaseOperatorShares {
        operator: Addr,
        staker: Addr,
        strategy: Addr,
        shares: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    IsDelegated { staker: Addr },
    IsOperator { operator: Addr },
    OperatorDetails { operator: Addr },
    DelegationApprover { operator: Addr },
    StakerOptOutWindowBlocks { operator: Addr },
    GetOperatorShares { operator: Addr, strategies: Vec<Addr> },
    GetDelegatableShares { staker: Addr },
    GetWithdrawalDelay { strategies: Vec<Addr> },
    CalculateWithdrawalRoot { withdrawal: Withdrawal },
    CurrentStakerDelegationDigestHash {current_staker_digest_hash_params: CurrentStakerDigestHashParams },
    StakerDelegationDigestHash { staker_digest_hash_params: StakerDigestHashParams },
    DelegationApprovalDigestHash { approver_digest_hash_params: ApproverDigestHashParams },
}

#[cw_serde]
pub struct OperatorDetails {
    pub __deprecated_earnings_receiver: Addr,
    pub delegation_approver: Addr,
    pub staker_opt_out_window_blocks: u64,
}

#[cw_serde]
pub struct QueuedWithdrawalParams {
    pub staker: Addr,
    pub strategies: Vec<Addr>,
    pub shares: Vec<u128>,
}

#[cw_serde]
pub struct SignatureWithExpiry {
    pub signature: Binary,
    pub expiry: u64,
}