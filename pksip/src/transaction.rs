#![warn(missing_docs)]
//! Transaction Layer.

use std::time::Duration;

pub use client::invite::ClientInviteTx;
pub use client::non_invite::ClientNonInviteTx;
pub use server::invite::ServerInviteTx;
pub use server::non_invite::ServerNonInviteTx;

pub mod key;
pub mod sip_transaction;
pub(crate) mod client;
pub(crate) mod manager;
pub(crate) mod server;

pub use client::ClientTx;
pub use server::ServerTx;

pub use manager::TransactionLayer;

/// Estimated round‑trip time (RTT) for message exchanges.
const T1: Duration = Duration::from_millis(500);

/// Maximum retransmission interval for non‑INVITE requests and INVITE responses.
const T2: Duration = Duration::from_secs(4);

/// Maximum duration that a message may remain in the network before being discarded.
const T4: Duration = Duration::from_secs(5);
