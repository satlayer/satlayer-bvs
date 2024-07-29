use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("DelegationManager.set_operator_details: stakerOptOutWindowBlocks cannot be > MAX_STAKER_OPT_OUT_WINDOW_BLOCKS")]
    InvalidInput {},

    #[error("DelegationManager.set_operator_details: stakerOptOutWindowBlocks cannot be decreased")]
    CannotBeDecreased {},

    #[error("DelegationManager.delegate: approver signature expired")]
    ApproverSignatureExpired {},

    #[error("DelegationManager.delegate: approver salt spent")]
    ApproverSaltSpent {},

    #[error("Signature Expired")]
    SignatureExpired {},

    #[error("Invalid Signature")]
    InvalidSignature {},

    #[error("Strategy Not Whitelisted")]
    StrategyNotWhitelisted {},

    #[error("Invalid Shares")]
    InvalidShares {},

    #[error("Max Strategy List Length Exceeded")]
    MaxStrategyListLengthExceeded {},

    #[error("Third Transfers Disabled")]
    ThirdTransfersDisabled {},

    #[error("Strategy Not Found")]
    StrategyNotFound {},
}
