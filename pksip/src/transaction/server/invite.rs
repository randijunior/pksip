use std::{
    cmp,
    ops::Deref,
    sync::{Arc, Mutex},
};

use futures_util::future::{
    Either, {self},
};
use tokio::{
    pin,
    sync::oneshot,
    time::{self},
};

use crate::{
    endpoint::Endpoint,
    error::Result,
    message::{CodeClass, SipMethod},
    transaction::{sip_transaction::Transaction, T1, T2, T4},
    transport::{IncomingRequest, OutgoingResponse},
};

type TxConfirmed = Arc<Mutex<Option<oneshot::Sender<()>>>>;
type RxConfirmed = oneshot::Receiver<()>;

/// Represents a Server INVITE transaction.
#[derive(Clone)]
pub struct ServerInviteTx {
    transaction: Transaction,
    pub(crate) received_provisional_tx: TxConfirmed,
}

impl ServerInviteTx {
    pub(crate) fn new(endpoint: &Endpoint, request: &IncomingRequest) -> Self {
        let transactions = endpoint.transactions();
        let method = request.message.method();

        assert!(
            matches!(method, SipMethod::Invite),
            "Expected SipMethod::Invite for server INVITE transaction, but got: {}",
            method
        );

        let transaction = Transaction::create_server(request, endpoint).unwrap();
        let uas_inv = Self {
            transaction,
            received_provisional_tx: Default::default(),
        };

        transactions.add_server_inv_to_map(uas_inv.clone());

        uas_inv
    }

    /// TODO: doc
    pub async fn respond(&self, response: OutgoingResponse) -> Result<()> {
        /* 
        let class = response.message.status_line.code.class();
        self.send_response(response).await?;

        match class {
            CodeClass::Provisional => {
                self.set_state(TransactionState::Proceeding);
            }
            CodeClass::Success => {
                self.on_terminated();
            }
            _ => {
                self.set_state(TransactionState::Completed);

                let (tx, rx) = oneshot::channel();

                self.received_provisional_tx
                    .lock()
                    .expect("Lock failed")
                    .replace(tx);
                self.retrans_loop(rx);
            }
        };
        */

        Ok(())
    }

    fn retrans_loop(&self, mut rx_confirmed: RxConfirmed) {

        /* 
        let unreliable = !self.is_reliable();
        let uas = self.clone();

        tokio::spawn(async move {
            let timer_h = time::sleep(64 * T1);
            let timer_g = if unreliable {
                Either::Left(time::sleep(T1))
            } else {
                Either::Right(future::pending::<()>())
            };

            pin!(timer_h);
            pin!(timer_g);

            'retrans: loop {
                tokio::select! {
                    _ = &mut timer_g => {
                        match uas.retransmit().await {
                            Ok(retrans) =>  {
                                let retrans = T1 * (1 << retrans);
                                let interval = cmp::min(retrans, T2);
                                let sleep = time::sleep(interval);
                                timer_g.set(Either::Left(sleep));
                            },
                            Err(err) =>  {
                                log::info!("Failed to retransmit: {}", err);
                            },
                        }
                     },
                    _ = &mut timer_h => {
                        // Timer H Expired!
                        uas.on_terminated();
                        break 'retrans;
                    }
                    _ = &mut rx_confirmed => {
                        // Got confirmed state!;
                        break 'retrans;
                    }
                }
            }
        });
        */
    }

    pub(crate) fn terminate(&self) {
        /* 
        if self.is_reliable() {
            self.on_terminated();
        } else {
            self.schedule_termination(T4);
        }
        */
    }
}

impl Deref for ServerInviteTx {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;
    use crate::{message::StatusCode};

    /*

    async fn tsx_uas_params<'a>() -> (Endpoint, IncomingRequest) {
        let endpoint = mock::default_endpoint();
        let request = mock::request(SipMethod::Invite);

        (endpoint, request)
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = ServerInviteTx::new(&endpoint, &mut request);
        let response = mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.get_state() == TransactionState::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_180_ringing() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = ServerInviteTx::new(&endpoint, &mut request);
        let response = mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        let response = mock::response(StatusCode::Ringing);
        tsx.respond(response).await.unwrap();

        assert!(tsx.get_state() == TransactionState::Proceeding);
    }

    #[tokio::test(start_paused = true)]
    async fn test_invite_timer_g_retransmission() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = ServerInviteTx::new(&endpoint, &mut request);

        let response = mock::response(StatusCode::BusyHere);
        tsx.respond(response).await.unwrap();

        time::sleep(T1 + Duration::from_millis(1)).await;
        assert!(tsx.retrans_count() == 1);

        time::sleep(T1 * 2 + Duration::from_millis(1)).await;
        assert!(tsx.retrans_count() == 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_h_expiration() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = ServerInviteTx::new(&endpoint, &mut request);

        let response = mock::response(StatusCode::BusyHere);

        tsx.respond(response).await.unwrap();

        time::sleep(T1 * 64 + Duration::from_millis(1)).await;
        assert!(tsx.get_state() == TransactionState::Terminated);
    }
     */
}
