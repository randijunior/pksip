#![warn(missing_docs)]
//! Transaction Layer.

use std::{
    time::{Duration},
};

pub use client::ClientTransaction;
pub use manager::TransactionManager;
pub use server::ServerTransaction;


use crate::transport::{
    IncomingRequest, IncomingResponse
};

pub(crate) mod client;
pub(crate) mod manager;
pub(crate) mod server;



#[cfg(test)]
mod tests;

#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub enum Role {
    UAS,
    UAC,
}

/// Estimated round‑trip time (RTT) for message exchanges.
pub(crate) const T1: Duration = Duration::from_millis(500);

/// Maximum retransmission interval for non‑INVITE requests and INVITE responses.
pub(crate) const T2: Duration = Duration::from_secs(4);

/// Maximum duration that a message may remain in the network before being discarded.
pub(crate) const T4: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
/// Defines the possible states of a SIP Transaction.
pub enum TransactionState {
    #[default]
    /// Initial state
    Initial,
    /// Calling state
    Calling,
    /// Trying state
    Trying,
    /// Proceeding state
    Proceeding,
    /// Completed state
    Completed,
    /// Confirmed state
    Confirmed,
    /// Terminated state
    Terminated,
}

impl std::fmt::Display for TransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = match self {
            Self::Initial => "Initial",
            Self::Calling => "Calling",
            Self::Trying => "Trying",
            Self::Proceeding => "Proceeding",
            Self::Completed => "Completed",
            Self::Confirmed => "Confirmed",
            Self::Terminated => "Terminated",
        };
        write!(f, "{}", state_str)
    }
}

#[derive(Clone)]
pub enum TransactionMessage {
    Request(IncomingRequest),
    Response(IncomingResponse),
}
