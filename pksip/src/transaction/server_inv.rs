use crate::{
    endpoint::Endpoint,
    error::Result,
    message::{SipMethod, REASON_TRYING},
    transaction::{SipTransaction, State, Transaction},
    transport::{IncomingRequest, OutgoingResponse},
};
use async_trait::async_trait;
use futures_util::future::{self, Either};
use std::{
    cmp,
    ops::Deref,
    sync::{Arc, Mutex},
};
use tokio::{
    pin,
    sync::oneshot,
    time::{self},
};

type TxConfirmed = Arc<Mutex<Option<oneshot::Sender<()>>>>;
type RxConfirmed = oneshot::Receiver<()>;

/// Represents a Server INVITE transaction.
#[derive(Clone)]
pub struct TsxUasInv {
    transaction: Transaction,
    pub(super) tx_confirmed: TxConfirmed,
}

impl TsxUasInv {
    pub(crate) async fn try_new(endpoint: &Endpoint, request: &mut IncomingRequest<'_>) -> Result<Self> {
        let tsx_layer = endpoint.get_tsx_layer();
        let method = request.msg.method();

        assert!(
            matches!(method, SipMethod::Invite),
            "Expected SipMethod::Invite for server INVITE transaction, but got: {}",
            method
        );

        let transaction = Transaction::create_uas_inv(request, endpoint);
        let tx_confirmed = Default::default();

        let uas_inv = TsxUasInv {
            transaction,
            tx_confirmed,
        };

        tsx_layer.add_server_tsx_inv_to_map(uas_inv.clone());
        request.set_tsx_inv(uas_inv.clone());

        // Generate a 100 (Trying) response.
        let response = endpoint.new_response(request, 100, REASON_TRYING);
        endpoint.send_response(&response).await?;

        Ok(uas_inv)
    }

    pub async fn respond(&self, response: &mut OutgoingResponse<'_>) -> Result<()> {
        self.tsx_send_response(response).await?;

        let code = response.status_code().into_i32();

        match code {
            200..=299 => {
                self.on_terminated();
            }
            300..=699 => {
                self.change_state_to(State::Completed);

                let (tx, rx) = oneshot::channel();

                self.tx_confirmed.lock().expect("Lock failed").replace(tx);
                self.initiate_retransmission(rx);
            }
            // Ignore probably provisional status code.
            _ => (),
        };

        Ok(())
    }

    fn initiate_retransmission(&self, mut rx_confirmed: RxConfirmed) {
        let unreliable = !self.reliable();
        let uas = self.clone();

        tokio::spawn(async move {
            let timer_h = time::sleep(64 * Self::T1);
            let timer_g = if unreliable {
                Either::Left(time::sleep(Self::T1))
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
                                let retrans = Self::T1 * (1 << retrans);
                                let interval = cmp::min(retrans, Self::T2);
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
}

//The TU passes any number of provisional responses to the
// server transaction.
#[async_trait]
impl SipTransaction for TsxUasInv {
    fn terminate(&self) {
        if self.reliable() {
            self.on_terminated();
        } else {
            self.schedule_termination(Self::T4);
        }
    }
}

impl Deref for TsxUasInv {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{message::StatusCode, transaction::mock};
    use tokio::time::Duration;

    async fn tsx_uas_params<'a>() -> (Endpoint, IncomingRequest<'a>) {
        let endpoint = mock::default_endpoint().await;
        let request = mock::request(SipMethod::Invite);

        (endpoint, request)
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::try_new(&endpoint, &mut request)
            .await
            .expect("should not fail");
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_180_ringing() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::try_new(&endpoint, &mut request)
            .await
            .expect("should not fail");
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 100);

        let response = &mut mock::response(StatusCode::Ringing);
        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 180);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test(start_paused = true)]
    async fn test_invite_timer_g_retransmission() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::try_new(&endpoint, &mut request)
            .await
            .expect("should not fail");

        let response = &mut mock::response(StatusCode::BusyHere);
        tsx.respond(response).await.unwrap();

        time::sleep(TsxUasInv::T1 + Duration::from_millis(1)).await;
        assert!(tsx.retrans_count() == 1);

        time::sleep(TsxUasInv::T1 * 2 + Duration::from_millis(1)).await;
        assert!(tsx.retrans_count() == 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_h_expiration() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::try_new(&endpoint, &mut request)
            .await
            .expect("should not fail");

        let response = &mut mock::response(StatusCode::BusyHere);

        tsx.respond(response).await.unwrap();

        time::sleep(TsxUasInv::T1 * 64 + Duration::from_millis(1)).await;
        assert!(tsx.get_state() == State::Terminated);
    }
}
