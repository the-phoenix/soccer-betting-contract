use cosmwasm_std::{Addr, OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("unauthorized sender: {sender}")]
    Unauthorized { sender: Addr },

    #[error("field cannot be empty: {field}")]
    EmptyField { field: &'static str },

    #[error("invalid fee bps")]
    InvalidFeeBps,

    #[error("invalid schedule")]
    InvalidSchedule,

    #[error("invalid stake denom, expected {expected}")]
    InvalidDenom { expected: String },

    #[error("exactly one coin of denom {expected} must be sent")]
    InvalidFunds { expected: String },

    #[error("zero stake is not allowed")]
    ZeroAmount,

    #[error("market not found: {market_id}")]
    MarketNotFound { market_id: u64 },

    #[error("betting is closed for market: {market_id}")]
    BettingClosed { market_id: u64 },

    #[error("market already settled: {market_id}")]
    MarketAlreadySettled { market_id: u64 },

    #[error("market already cancelled: {market_id}")]
    MarketAlreadyCancelled { market_id: u64 },

    #[error("market cannot be settled before kickoff: {market_id}")]
    BetTooEarlyToSettle { market_id: u64 },

    #[error("no winning bet for sender")]
    NoWinningBet,

    #[error("claim already submitted")]
    AlreadyClaimed,

    #[error("refund already submitted")]
    AlreadyRefunded,

    #[error("market is not cancelled: {market_id}")]
    MarketNotCancelled { market_id: u64 },
}
