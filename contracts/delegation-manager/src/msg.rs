use crate::utils::{
    CurrentStakerDigestHashParams, ExecuteDelegateParams, QueryApproverDigestHashParams,
    QueryStakerDigestHashParams, Withdrawal,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
        sender_public_key: Binary,
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
        approver_signature_and_expiry: ExecuteSignatureWithExpiry,
    },
    DelegateToBySignature {
        params: ExecuteDelegateParams,
        staker_public_key: String,
        staker_signature_and_expiry: ExecuteSignatureWithExpiry,
        approver_signature_and_expiry: ExecuteSignatureWithExpiry,
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
        middleware_times_index: Uint64,
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
        shares: Uint128,
    },
    DecreaseDelegatedShares {
        staker: Addr,
        strategy: Addr,
        shares: Uint128,
    },
    SetMinWithdrawalDelayBlocks {
        new_min_withdrawal_delay_blocks: u64,
    },
    SetStrategyWithdrawalDelayBlocks {
        strategies: Vec<Addr>,
        withdrawal_delay_blocks: Vec<Uint64>,
    },
    TransferOwnership {
        new_owner: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IsDelegated { staker: Addr },

    #[returns(bool)]
    IsOperator { operator: Addr },

    #[returns(OperatorDetails)]
    OperatorDetails { operator: Addr },

    #[returns(Addr)]
    DelegationApprover { operator: Addr },

    #[returns(u64)]
    StakerOptOutWindowBlocks { operator: Addr },

    #[returns(Vec<Uint128>)]
    GetOperatorShares {
        operator: Addr,
        strategies: Vec<Addr>,
    },

    #[returns((Vec<Addr>, Vec<Uint128>))]
    GetDelegatableShares { staker: Addr },

    #[returns(Vec<u64>)]
    GetWithdrawalDelay { strategies: Vec<Addr> },

    #[returns(Binary)]
    CalculateWithdrawalRoot { withdrawal: Withdrawal },

    #[returns(Binary)]
    StakerDelegationDigestHash {
        staker_digest_hash_params: QueryStakerDigestHashParams,
    },

    #[returns(Binary)]
    DelegationApprovalDigestHash {
        approver_digest_hash_params: QueryApproverDigestHashParams,
    },

    #[returns(Binary)]
    CalculateCurrentStakerDelegationDigestHash {
        current_staker_digest_hash_params: CurrentStakerDigestHashParams,
    },

    #[returns(Uint128)]
    GetStakerNonce { staker: Addr },

    #[returns(Vec<(Addr, Uint128)>)]
    GetOperatorStakers { operator: Addr },

    #[returns(Uint128)]
    GetCumulativeWithdrawalsQueuedNonce { staker: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OperatorDetails {
    pub deprecated_earnings_receiver: Addr,
    pub delegation_approver: Addr,
    pub staker_opt_out_window_blocks: u64,
}

#[cw_serde]
pub struct QueuedWithdrawalParams {
    pub withdrawer: Addr,
    pub strategies: Vec<Addr>,
    pub shares: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SignatureWithExpiry {
    pub signature: Binary,
    pub expiry: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExecuteSignatureWithExpiry {
    pub signature: String,
    pub expiry: Uint64,
}
