use std::cmp;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use futures_util::future::Either;
use futures_util::future::{self};
use tokio::pin;
use tokio::sync::oneshot;
use tokio::time::{self};

use crate::message::SipMethod;
use crate::transaction::State;
use crate::transaction::Transaction;
use crate::transaction::T1;
use crate::transaction::T2;
use crate::transaction::T4;
use crate::transport::IncomingResponse;
use crate::transport::OutgoingRequest;
use crate::Result;
use crate::SipEndpoint;

type TxCompleted = Arc<Mutex<Option<oneshot::Sender<()>>>>;
type RxCompleted = oneshot::Receiver<()>;

/// Represents a Client Non INVITE transaction.
#[derive(Clone)]
pub struct ClientTransaction {
    transaction: Transaction,
    tx_completed: TxCompleted,
}

impl ClientTransaction {
    pub(crate) async fn send(mut request: OutgoingRequest, endpoint: &SipEndpoint) -> Result<()> {
        let transactions = endpoint.transactions();
        let method = request.msg.method();

        assert!(
            !matches!(method, SipMethod::Invite | SipMethod::Ack),
            "Invalid method for non-INVITE client transaction: expected non-INVITE/non-ACK, got: {}",
            method
        );

        let transaction = Transaction::new_uac(&request, endpoint);
        let (tx, rx) = oneshot::channel();

        let tx_completed = Arc::new(Mutex::new(Some(tx)));

        let uac = Self {
            transaction,
            tx_completed,
        };

        uac.tsx_send_request(&mut request).await?;
        uac.change_state_to(State::Trying);

        uac.retrans_loop(rx);

        transactions.add_client_tsx_to_map(uac);

        Ok(())
    }

    fn retrans_loop(&self, mut rx_completed: RxCompleted) {
        let unreliable = !self.reliable();
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
                                let interval = if state == State::Trying {
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
    }

    pub(crate) async fn receive(&self, response: &IncomingResponse) -> Result<bool> {
        let code = response.response.code();
        self.set_last_status_code(code);

        match self.get_state() {
            State::Trying if code.is_provisional() => {
                self.change_state_to(State::Proceeding);
            }
            State::Trying | State::Proceeding if code.is_final() => {
                self.change_state_to(State::Completed);

                let tx = self.tx_completed.lock().expect("Lock failed").take();
                if let Some(tx) = tx {
                    tx.send(()).unwrap();
                }
                self.terminate();
            }
            State::Completed => {
                self.retransmit().await?;

                return Ok(true);
            }
            _ => (),
        }

        Ok(false)
    }

    pub(crate) fn terminate(&self) {
        if self.reliable() {
            self.on_terminated();
        } else {
            // Start timer K
            self.schedule_termination(T4);
        }
    }
}

impl DerefMut for ClientTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for ClientTransaction {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;
    use tokio::time::{self};

    use super::*;
    use crate::message::SipMethod;
    use crate::message::StatusCode;
    use crate::transaction::mock;

    /*

    #[tokio::test]
    async fn test_entered_trying() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = ClientTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);
    }

    #[tokio::test(start_paused = true)]
    async fn test_fire_timer_f() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = ClientTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);

        // Wait for the timer to fire
        time::sleep(ClientTransaction::T1 * 64 + Duration::from_millis(1)).await;

        assert_eq!(uac.get_state(), State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_fire_timer_k() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);
        let response = mock::incoming_response(StatusCode::Ok);

        let uac = ClientTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);

        uac.receive(&response).await.unwrap();
        // Wait for the timer to fire
        time::sleep(ClientTransaction::T4 + Duration::from_millis(1)).await;

        assert_eq!(uac.get_state(), State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_e_retransmission() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = ClientTransaction::send(request, &endpoint).await.unwrap();

        assert!(uac.retrans_count() == 0);

        // For the default values of T1 and T2, this results in
        // intervals of 500 ms, 1 s, 2 s, 4 s, 4 s, 4 s, etc.
        assert_eq!(uac.get_state(), State::Trying);
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

        assert_eq!(uac.get_state(), State::Trying);
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = ClientTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);

        let response = mock::incoming_response(100.into());
        uac.receive(&response).await.unwrap();

        assert_eq!(uac.get_state(), State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);
        let response = mock::incoming_response(StatusCode::Ok);

        let uac = ClientTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);

        uac.receive(&response).await.unwrap();

        assert!(uac.last_status_code().unwrap().as_u16() == 200);
        assert!(uac.get_state() == State::Completed);
    }
    */
}
