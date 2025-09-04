use std::cmp;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;

use futures_util::future::Either;
use futures_util::future::{self};
use tokio::pin;
use tokio::sync::oneshot;
use tokio::time::{self};

use crate::core::SipEndpoint;
use crate::error::Result;
use crate::message::CodeClass;
use crate::message::SipMethod;
use crate::transaction::State;
use crate::transaction::Transaction;
use crate::transaction::T1;
use crate::transaction::T2;
use crate::transaction::T4;
use crate::transport::IncomingRequest;
use crate::transport::OutgoingResponse;

type TxConfirmed = Arc<Mutex<Option<oneshot::Sender<()>>>>;
type RxConfirmed = oneshot::Receiver<()>;

/// Represents a Server INVITE transaction.
#[derive(Clone)]
pub struct ServerInvTransaction {
    transaction: Transaction,
    pub(crate) tx_confirmed: TxConfirmed,
}

impl ServerInvTransaction {
    pub(crate) fn new(endpoint: &SipEndpoint, request: &IncomingRequest) -> Self {
        let transactions = endpoint.transactions();
        let method = request.method();

        assert!(
            matches!(method, SipMethod::Invite),
            "Expected SipMethod::Invite for server INVITE transaction, but got: {}",
            method
        );

        let transaction = Transaction::new_uas_inv(request, endpoint);
        let uas_inv = Self {
            transaction,
            tx_confirmed: Default::default(),
        };

        transactions.add_server_inv_to_map(uas_inv.clone());

        uas_inv
    }

    /// TODO: doc
    pub async fn respond(&self, response: &mut OutgoingResponse) -> Result<()> {
        self.tsx_send_response(response).await?;

        match response.status_code().class() {
            CodeClass::Provisional => {
                self.change_state_to(State::Proceeding);
            }
            CodeClass::Success => {
                self.on_terminated();
            }
            _ => {
                self.change_state_to(State::Completed);

                let (tx, rx) = oneshot::channel();

                self.tx_confirmed.lock().expect("Lock failed").replace(tx);
                self.retrans_loop(rx);
            }
        };

        Ok(())
    }

    fn retrans_loop(&self, mut rx_confirmed: RxConfirmed) {
        let unreliable = !self.reliable();
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
    }

    pub(crate) fn terminate(&self) {
        if self.reliable() {
            self.on_terminated();
        } else {
            self.schedule_termination(T4);
        }
    }
}

impl Deref for ServerInvTransaction {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;
    use crate::message::StatusCode;
    use crate::transaction::mock;

    async fn tsx_uas_params<'a>() -> (SipEndpoint, IncomingRequest) {
        let endpoint = mock::default_endpoint().await;
        let request = mock::request(SipMethod::Invite);

        (endpoint, request)
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = ServerInvTransaction::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().as_u16() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_180_ringing() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = ServerInvTransaction::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().as_u16() == 100);

        let response = &mut mock::response(StatusCode::Ringing);
        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().as_u16() == 180);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test(start_paused = true)]
    async fn test_invite_timer_g_retransmission() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = ServerInvTransaction::new(&endpoint, &mut request);

        let response = &mut mock::response(StatusCode::BusyHere);
        tsx.respond(response).await.unwrap();

        time::sleep(T1 + Duration::from_millis(1)).await;
        assert!(tsx.retrans_count() == 1);

        time::sleep(T1 * 2 + Duration::from_millis(1)).await;
        assert!(tsx.retrans_count() == 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_h_expiration() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = ServerInvTransaction::new(&endpoint, &mut request);

        let response = &mut mock::response(StatusCode::BusyHere);

        tsx.respond(response).await.unwrap();

        time::sleep(T1 * 64 + Duration::from_millis(1)).await;
        assert!(tsx.get_state() == State::Terminated);
    }
}
