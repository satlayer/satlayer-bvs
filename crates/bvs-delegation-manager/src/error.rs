use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("DelegationManager: Unauthorized")]
    Unauthorized {},

    #[error("DelegationManager.set_operator_details: stakerOptOutWindowBlocks cannot be > MAX_STAKER_OPT_OUT_WINDOW_BLOCKS")]
    CannotBeExceedMaxStakerOptOutWindowBlocks {},

    #[error("DelegationManager._set_strategy_withdrawal_delay_blocks: withdrawalDelayBlocks cannot be > MAX_WITHDRAWAL_DELAY_BLOCKS")]
    CannotBeExceedMaxWithdrawalDelayBlocks {},

    #[error("DelegationManager._set_min_withdrawal_delay_blocks: minWithdrawalDelayBlocks cannot be > MAX_WITHDRAWAL_DELAY_BLOCKS")]
    MinCannotBeExceedMaxWithdrawalDelayBlocks {},

    #[error("DelegationManager: input length mismatch")]
    InputLengthMismatch {},

    #[error(
        "DelegationManager.set_operator_details: stakerOptOutWindowBlocks cannot be decreased"
    )]
    CannotBeDecreased {},

    #[error("DelegationManager.delegate: approver signature expired")]
    ApproverSignatureExpired {},

    #[error("DelegationManager.delegate: approver salt spent")]
    ApproverSaltSpent {},

    #[error("DelegationManager.delegate_to: staker already delegated")]
    StakerAlreadyDelegated {},

    #[error("DelegationManager: operator not registered")]
    OperatorNotRegistered {},

    #[error("DelegationManager.delegate_to_by_signature: staker signature expired")]
    StakerSignatureExpired {},

    #[error("DelegationManager.delegate_to_by_signature: nonce overflow")]
    NonceOverflow,

    #[error("DelegationManager._decrease_operator_shares: underflow")]
    Underflow,

    #[error(
        "DelegationManager._remove_shares_and_queue_withdrawal: staker cannot be zero address"
    )]
    CannotBeZero {},

    #[error("DelegationManager._remove_shares_and_queue_withdrawal: strategies cannot be empty")]
    CannotBeEmpty {},

    #[error("DelegationManager._removeSharesAndQueueWithdrawal: withdrawer must be same address as staker if thirdPartyTransfersForbidden are set")]
    MustBeSameAddress {},

    #[error("DelegationManager._completeQueuedWithdrawal: action is not in queue")]
    ActionNotInQueue {},

    #[error("DelegationManager._completeQueuedWithdrawal: minWithdrawalDelayBlocks period has not yet passed")]
    MinWithdrawalDelayNotPassed {},

    #[error("DelegationManager._completeQueuedWithdrawal: withdrawalDelayBlocks period has not yet passed for this strategy")]
    StrategyWithdrawalDelayNotPassed {},

    #[error("DelegationManager.queueWithdrawal: withdrawer must be staker")]
    WithdrawerMustBeStaker {},

    #[error("DelegationManager.undelegate: staker must be delegated to undelegate")]
    StakerNotDelegated {},

    #[error("DelegationManager.undelegate: operators cannot be undelegated")]
    OperatorCannotBeUndelegated {},

    #[error("DelegationManager._delegate: invalid signature")]
    InvalidSignature {},

    #[error("DelegationManager.increase_delegated_shares: not delegated")]
    NotDelegated {},
}
