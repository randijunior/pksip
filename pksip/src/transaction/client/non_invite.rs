use std::{
    cmp,
    ops::{Deref, DerefMut},
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
    Result, Endpoint,
    message::{SipMethod, Request},
    transaction::{T1, T2, T4, sip_transaction::Transaction},
    transport::{IncomingResponse, OutgoingMessageInfo, OutgoingRequest},
};

type TxCompleted = Arc<Mutex<Option<oneshot::Sender<()>>>>;
type RxCompleted = oneshot::Receiver<()>;

/// Represents a Client Non INVITE transaction.
#[derive(Clone)]
pub struct ClientNonInviteTx {
    transaction: Transaction,
    tx_completed: TxCompleted,
}

impl ClientNonInviteTx {
    pub async fn send_request(
        endpoint: &Endpoint,
        request: Request,
        target: Option<OutgoingMessageInfo>,
    ) -> Result<Self> {
        // let transactions = endpoint.transactions();
        // let method = request.message.method();

        // assert!(
        //     !matches!(method, SipMethod::Invite | SipMethod::Ack),
        //     "Invalid method for non-INVITE client transaction: expected non-INVITE/non-ACK, got: {}",
        //     method
        // );

        // let transaction = Transaction::new_client(method, &endpoint);
        // let (tx, rx) = oneshot::channel();

        // let tx_completed = Arc::new(Mutex::new(Some(tx)));

        // let uac = Self {
        //     transaction,
        //     tx_completed,
        // };

        // uac.send_request(&mut request).await?;
        // uac.set_state(TransactionState::Trying);

        // uac.retrans_loop(rx);

        // transactions.add_client_tsx_to_map(uac);

        todo!()
    }

    fn retrans_loop(&self, mut rx_completed: RxCompleted) {
        /*
        let unreliable = !self.is_reliable();
        let uac = self.clone();

        tokio::spawn(async move {
            pin! {
                let timer_f = time::sleep(64 * T1);
                let timer_e = if unreliable {
                    Either::Left(time::sleep(T1))
                } else {
                    Either::Right(future::pending::<()>())
                };
            }

            'retrans: loop {
                tokio::select! {
                    _ = &mut timer_e => {
                        let state = uac.get_state();
                        match uac.retransmit().await {
                            Ok(retrans) =>  {
                                let interval = if state == TransactionState::Trying {
                                    let retrans = T1 * (1 << retrans);
                                    cmp::min(retrans, T2)
                                } else {
                                    T2
                                };
                                let sleep = time::sleep(interval);
                                timer_e.set(Either::Left(sleep));
                            },
                            Err(err) =>  {
                                log::info!("Failed to retransmit: {}", err);
                            },
                        }
                    }
                    _ = &mut timer_f => {
                        // Timer F Expired!
                        uac.on_terminated();
                        break 'retrans;
                    }

                    _ = &mut rx_completed => {
                        // Got completed state!;
                        break 'retrans;
                    }
                }
            }
        });
        */
    }

    pub(crate) async fn receive(&self, response: &IncomingResponse) -> Result<bool> {
        /*
                let code = response.message.code();

                match self.get_state() {
                    TransactionState::Trying if code.is_provisional() => {
                        self.set_state(TransactionState::Proceeding);
                    }
                    TransactionState::Trying | TransactionState::Proceeding if code.is_final() => {
                        self.set_state(TransactionState::Completed);

                        let tx = self.tx_completed.lock().expect("Lock failed").take();
                        if let Some(tx) = tx {
                            tx.send(()).unwrap();
                        }
                        self.terminate();
                    }
                    TransactionState::Completed => {
                        self.retransmit().await?;

                        return Ok(true);
                    }
                    _ => (),
                }
        */
        Ok(false)
    }

    pub(crate) fn terminate(&self) {
        /*
        if self.is_reliable() {
            self.on_terminated();
        } else {
            // Start timer K
            self.schedule_termination(T4);
        }
        */
    }
}

impl DerefMut for ClientNonInviteTx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for ClientNonInviteTx {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{
        Duration, {self},
    };

    use super::*;
    use crate::message::{SipMethod, StatusCode};

    /*

    #[tokio::test]
    async fn test_entered_trying() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = ClientNonInviteTx::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), TransactionState::Trying);
    }

    #[tokio::test(start_paused = true)]
    async fn test_fire_timer_f() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = ClientNonInviteTx::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), TransactionState::Trying);

        // Wait for the timer to fire
        time::sleep(ClientNonInviteTx::T1 * 64 + Duration::from_millis(1)).await;

        assert_eq!(uac.get_state(), TransactionState::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_fire_timer_k() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Options);
        let response = mock::incoming_response(StatusCode::Ok);

        let uac = ClientNonInviteTx::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), TransactionState::Trying);

        uac.receive(&response).await.unwrap();
        // Wait for the timer to fire
        time::sleep(ClientNonInviteTx::T4 + Duration::from_millis(1)).await;

        assert_eq!(uac.get_state(), TransactionState::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_e_retransmission() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = ClientNonInviteTx::send(request, &endpoint).await.unwrap();

        assert!(uac.retrans_count() == 0);

        // For the default values of T1 and T2, this results in
        // intervals of 500 ms, 1 s, 2 s, 4 s, 4 s, 4 s, etc.
        assert_eq!(uac.get_state(), TransactionState::Trying);
        // 500 ms
        time::sleep(Duration::from_millis(500 + 1)).await;
        assert!(uac.retrans_count() == 1);
        // 1 s
        time::sleep(Duration::from_secs(1) + Duration::from_millis(1)).await;
        assert!(uac.retrans_count() == 2);
        // 2 s
        time::sleep(Duration::from_secs(2) + Duration::from_millis(1)).await;
        assert!(uac.retrans_count() == 3);
        // 4s
        time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
        assert!(uac.retrans_count() == 4);
        // 4s
        time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
        assert!(uac.retrans_count() == 5);
        // 4s
        time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
        assert!(uac.retrans_count() == 6);

        assert_eq!(uac.get_state(), TransactionState::Trying);
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = ClientNonInviteTx::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), TransactionState::Trying);

        let response = mock::incoming_response(100.into());
        uac.receive(&response).await.unwrap();

        assert_eq!(uac.get_state(), TransactionState::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Options);
        let response = mock::incoming_response(StatusCode::Ok);

        let uac = ClientNonInviteTx::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), TransactionState::Trying);

        uac.receive(&response).await.unwrap();

        assert!(uac.last_status_code().unwrap().as_u16() == 200);
        assert!(uac.get_state() == TransactionState::Completed);
    }
    */
}
