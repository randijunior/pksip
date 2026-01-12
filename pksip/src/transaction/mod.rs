#![warn(missing_docs)]
//! Transaction Layer.

use std::time::Duration;

pub use client::ClientTransaction;
pub use manager::TransactionManager;
pub use server::ServerTransaction;
use tokio::sync::mpsc;

use crate::transport::{IncomingRequest, IncomingResponse};

pub(crate) mod client;
pub(crate) mod manager;
pub(crate) mod server;
pub(crate) mod fsm;

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



#[derive(Clone)]
pub enum TransactionMessage {
    Request(IncomingRequest),
    Response(IncomingResponse),
}

struct PeekableReceiver<T> {
    rx: mpsc::Receiver<T>,
    peeked: Option<T>
}

impl<T> From<mpsc::Receiver<T>> for PeekableReceiver<T> {
    fn from(rx: mpsc::Receiver<T>) -> Self {
        Self::new(rx)
    }
}

impl<T> PeekableReceiver<T> {
    pub fn new(rx: mpsc::Receiver<T>) -> Self {
        Self { rx, peeked: None }
    }

    pub async fn recv(&mut self) -> Option<T> {
        match self.peeked.take() {
            Some(msg) => Some(msg),
            None => self.rx.recv().await,
        }
    }
    pub fn try_recv(&mut self) -> std::result::Result<T, mpsc::error::TryRecvError> {
        match self.peeked.take() {
            Some(msg) => Ok(msg),
            None => self.rx.try_recv(),
        }
    }
    pub async fn peek(&mut self) -> Option<&T> {
        if self.peeked.is_none() {
            self.peeked = self.rx.recv().await;
        }
        self.peeked.as_ref()
    }

    pub async fn recv_if(&mut self, func: impl FnOnce(&T) -> bool) -> Option<T> {
        match self.peek().await {
            Some(matched) if func(matched) => {
                self.peeked.take()
            }
            _ => None,
        }
    }
}