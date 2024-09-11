use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("StrategyFactory.reply: unknown reply id")]
    UnknownReplyId {},

    #[error("StrategyFactory.handle_instantiate_reply: missing instantiate data")]
    MissingInstantiateData {},

    #[error("StrategyFactory.handle_instantiate_reply: instantiate error")]
    InstantiateError {},

    #[error("StrategyFactory.update_config: unauthorized")]
    Unauthorized {},

    #[error("StrategyFactory.blacklist_tokens: token already blacklisted")]
    TokenAlreadyBlacklisted {},
}
