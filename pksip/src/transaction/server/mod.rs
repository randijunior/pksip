pub mod invite;
pub mod non_invite;

use invite::ServerInviteTx;
use non_invite::ServerNonInviteTx;

use crate::{transaction::{key::TransactionKey, sip_transaction::TransactionState}, transport::{IncomingRequest, OutgoingResponse}};

#[derive(Clone)]
/// An Server Transaction, either Invite or NonInvite.
pub enum ServerTx {
    /// An NonInvite Server Transaction.
    NonInvite(ServerNonInviteTx),
    /// An Invite Server Transaction.
    Invite(ServerInviteTx),
}

/* 
impl ServerTx {
    pub async fn respond(&self, msg: OutgoingResponse) -> Result<()> {
        match self {
            ServerTx::NonInvite(uas) => uas.respond(msg).await,
            ServerTx::Invite(uas_inv) => uas_inv.respond(msg).await,
        }
    }

    pub(crate) fn transaction_key(&self) -> &TransactionKey {
        match self {
            ServerTx::NonInvite(uas) => uas.key(),
            ServerTx::Invite(uas_inv) => uas_inv.key(),
        }
    }

    pub(crate) async fn receive_request(&self, request: &IncomingRequest) -> Result<()> {
        match self {
            ServerTx::NonInvite(uas) => {
                if matches!(
                    uas.get_state(),
                    TransactionState::Proceeding | TransactionState::Completed
                ) {
                    uas.retransmit().await?;
                }
                Ok(())
            }
            ServerTx::Invite(uas_inv) => {
                match uas_inv.get_state() {
                    TransactionState::Completed if request.message.method() == SipMethod::Ack => {
                        uas_inv.set_state(TransactionState::Confirmed);
                        let mut lock = uas_inv.tx_confirmed.lock().expect("Lock failed");
                        if let Some(sender) = lock.take() {
                            sender.send(()).unwrap();
                        }
                        drop(lock);
                        uas_inv.terminate();
                    }
                    TransactionState::Proceeding => {
                        uas_inv.retransmit().await?;
                    }
                    _ => (),
                }
                Ok(())
            }
        }
    }
}
*/
impl From<ServerNonInviteTx> for ServerTx {
    fn from(tsx: ServerNonInviteTx) -> Self {
        ServerTx::NonInvite(tsx)
    }
}

impl From<ServerInviteTx> for ServerTx {
    fn from(tsx: ServerInviteTx) -> Self {
        ServerTx::Invite(tsx)
    }
}
