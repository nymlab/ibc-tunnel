use thiserror::Error;

use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;

use cosmwasm_tunnel::ChannelError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),

    #[error("{0}")]
    ChannelError(#[from] ChannelError),

    #[error("Cannot register over an existing channel")]
    ChannelAlreadyRegistered,

    #[error("Invalid reply id")]
    InvalidReplyId,

    #[error("Invalid Connection Id")]
    InvalidConnectionId,

    #[error("Unauthorized")]
    Unauthorized,
}
