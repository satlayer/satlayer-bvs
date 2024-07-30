use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("DelegationManager._only_owner: Unauthorized")]
    Unauthorized {},

    #[error("DelegationManager.set_operator_details: stakerOptOutWindowBlocks cannot be > MAX_STAKER_OPT_OUT_WINDOW_BLOCKS")]
    CannotBeExceedMAXSTAKEROPTOUTWINDOWBLOCKS {},

    #[error("DelegationManager._set_strategy_withdrawal_delay_blocks: withdrawalDelayBlocks cannot be > MAX_WITHDRAWAL_DELAY_BLOCKS")]
    CannotBeExceedMAXWITHDRAWALDELAYBLOCKS {},

    #[error("DelegationManager._set_min_withdrawal_delay_blocks: minWithdrawalDelayBlocks cannot be > MAX_WITHDRAWAL_DELAY_BLOCKS")]
    MinCannotBeExceedMAXWITHDRAWALDELAYBLOCKS {},

    #[error("DelegationManager._set_strategy_withdrawal_delay_blocks: input length mismatch")]
    InputLengthMismatch {},

    #[error("DelegationManager.set_operator_details: stakerOptOutWindowBlocks cannot be decreased")]
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
}
