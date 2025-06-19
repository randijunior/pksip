use std::{
    cmp,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use futures_util::future::{self, Either};
use tokio::{
    pin,
    sync::oneshot,
    time::{self},
};

use crate::{
    message::SipMethod,
    transaction::State,
    transport::{IncomingResponse, OutgoingRequest},
    Endpoint, Result,
};

use super::{SipTransaction, Transaction};

type TxCompleted = Arc<Mutex<Option<oneshot::Sender<()>>>>;
type RxCompleted = oneshot::Receiver<()>;

/// Represents a Client Non INVITE transaction.
#[derive(Clone)]
pub struct TsxUac {
    transaction: Transaction,
    tx_completed: TxCompleted,
}

impl TsxUac {
    pub(crate) async fn send(mut request: OutgoingRequest<'_>, endpoint: &Endpoint) -> Result<Self> {
        let tsx_layer = endpoint.get_tsx_layer();
        let method = request.msg.method();

        assert!(
            !matches!(method, SipMethod::Invite | SipMethod::Ack),
            "Invalid method for non-INVITE client transaction: expected non-INVITE/non-ACK, got: {}",
            method
        );

        let transaction = Transaction::create_uac(&request, endpoint);
        let (tx, rx) = oneshot::channel();

        let tx_completed = Arc::new(Mutex::new(Some(tx)));

        let uac = TsxUac {
            transaction,
            tx_completed,
        };

        uac.tsx_send_request(&mut request).await?;
        uac.change_state_to(State::Trying);

        tsx_layer.add_client_tsx_to_map(uac.clone());

        uac.initiate_retransmission(rx).await?;

        Ok(uac)
    }

    async fn initiate_retransmission(&self, mut rx_completed: RxCompleted) -> Result<()> {
        let unreliable = !self.reliable();
        let uac = self.clone();

        tokio::spawn(async move {
            pin! {
                let timer_f = time::sleep(64 * Self::T1);
                let timer_e = if unreliable {
                    Either::Left(time::sleep(Self::T1))
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
                                    let retrans = Self::T1 * (1 << retrans);
                                    cmp::min(retrans, Self::T2)
                                } else {
                                    Self::T2
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

        Ok(())
    }

    pub(crate) async fn receive(&self, response: &IncomingResponse<'_>) -> Result<bool> {
        let code = response.msg.code();
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
            State::Completed =>  {
                self.retransmit().await?;

                return Ok(true)
            },
            _=> ()
        }

        Ok(false)
    }
}

#[async_trait::async_trait]
impl SipTransaction for TsxUac {
    fn terminate(&self) {
        if self.reliable() {
            self.on_terminated();
        } else {
            // Start timer K
            self.schedule_termination(Self::T4);
        }
    }
}

impl DerefMut for TsxUac {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for TsxUac {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        message::{SipMethod, StatusCode},
        transaction::mock,
    };
    use tokio::time::{self, Duration};

    #[tokio::test]
    async fn test_entered_trying() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = TsxUac::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);
    }

    #[tokio::test(start_paused = true)]
    async fn test_fire_timer_f() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = TsxUac::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);

        // Wait for the timer to fire
        time::sleep(TsxUac::T1 * 64 + Duration::from_millis(1)).await;

        assert_eq!(uac.get_state(), State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_fire_timer_k() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);
        let response = mock::incoming_response(StatusCode::Ok);

        let uac = TsxUac::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);

        uac.receive(&response).await.unwrap();
        // Wait for the timer to fire
        time::sleep(TsxUac::T4 + Duration::from_millis(1)).await;

        assert_eq!(uac.get_state(), State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_e_retransmission() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Options);

        let uac = TsxUac::send(request, &endpoint).await.unwrap();

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

        let uac = TsxUac::send(request, &endpoint).await.unwrap();

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

        let uac = TsxUac::send(request, &endpoint).await.unwrap();

        assert_eq!(uac.get_state(), State::Trying);

        uac.receive(&response).await.unwrap();

        assert!(uac.last_status_code().unwrap().into_i32() == 200);
        assert!(uac.get_state() == State::Completed);
    }
}
