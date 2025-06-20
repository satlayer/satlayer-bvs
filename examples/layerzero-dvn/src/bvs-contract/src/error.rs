use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {msg}")]
    Unauthorized { msg: String },

    #[error("Packet already broadcasted")]
    PacketAlreadyExists,

    #[error("Packet verification already exists")]
    PacketVerificationExists,

    #[error("Packet already finalized")]
    PacketAlreadyFinalized,

    #[error("Packet does not have enough verifications")]
    InsufficientVerifications,
}
