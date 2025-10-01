use cosmwasm_std::StdError;
use thiserror::Error;

/// dd_blind_box 合约错误定义
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("No NFTs available for distribution")] 
    NoNftsAvailable,

    #[error("Invalid state for this action")] 
    InvalidState,

    #[error("Reveal phase not active")] 
    RevealNotActive,

    #[error("Commit phase not active")] 
    CommitNotActive,

    #[error("Nothing to reveal for this voter")] 
    NothingToReveal,

    #[error("Invalid state transition: from {from:?} to {to:?}")]
    InvalidStateTransition { from: crate::state::VoteState, to: crate::state::VoteState },

    #[error("Outside time window: current {current}, window {start}-{end}")]
    OutsideWindow { current: u64, start: u64, end: u64 },

    #[error("Too many voters: {count} exceeds maximum {max}")]
    TooManyVoters { count: usize, max: usize },
}


