use crate::{
    endpoint::Endpoint,
    error::Result,
    message::Method,
    transaction::{ServerTsx, SipTransaction, State, Transaction},
    transport::{IncomingRequest, OutgoingResponse},
};
use async_trait::async_trait;
use std::{cmp, ops::Deref};
use tokio::{
    sync::oneshot,
    time::{self, Instant},
};

type TxConfirmedState = std::sync::Arc<std::sync::Mutex<Option<oneshot::Sender<()>>>>;

/// Represents a Server INVITE transaction.
#[derive(Clone)]
pub struct TsxUasInv {
    tsx: Transaction,
    tx: TxConfirmedState,
}

impl TsxUasInv {
    pub(crate) fn new(endpoint: &Endpoint, request: &mut IncomingRequest) -> Self {
        assert!(
            matches!(request.method(), Method::Invite),
            "Request method must be Invite"
        );
        let tsx_layer = endpoint.get_tsx_layer();
        let tsx = Transaction::create_uas(request, endpoint);
        let server_tsx = TsxUasInv {
            tx: Default::default(),
            tsx,
        };

        {
            let server_tsx = server_tsx.clone();
            let server_tsx = ServerTsx::Invite(server_tsx);
            request.tsx = Some(server_tsx);
        }

        tsx_layer.new_server_inv_tsx(server_tsx.clone());

        server_tsx
    }

    fn start_retrans_timer(&self, mut confirmed_state: oneshot::Receiver<()>) {
        let reliable = self.reliable();
        let tsx = self.tsx.clone();

        tokio::spawn(async move {
            let timer_h = time::sleep(Self::T4);
            tokio::pin!(timer_h);
            if reliable {
                tokio::select! {
                    _ = &mut timer_h => {
                        // Timer H Expired!
                        tsx.set_state(State::Terminated);
                        tsx.on_terminated();
                        return;
                    }
                    _ = &mut confirmed_state => {
                        // Got confirmed state!;
                        return;
                    }
                }
            }
            let timer_g = time::sleep(Self::T1);
            tokio::pin!(timer_g);
            loop {
                tokio::select! {
                    _ = &mut timer_g => {
                        if let Ok(retrans_count) = tsx.retransmit().await {
                            let retrans = 2u32.pow(retrans_count);
                            let next_interval = cmp::min(Self::T1*retrans, Self::T2);
                            timer_g.as_mut().reset(Instant::now() + next_interval);
                        } else {
                            // Error retransmitting
                        }
                    }
                    _ = &mut timer_h => {
                        // Timer H Expired!
                        tsx.set_state(State::Terminated);
                        tsx.on_terminated();
                        return;
                    }
                    _ = &mut confirmed_state => {
                        // Got confirmed state!;
                        return;
                    }
                }
            }
        });
    }

    pub(crate) async fn recv_msg<'a>(&self, msg: &IncomingRequest<'a>) -> Result<()> {
        match self.get_state() {
            State::Completed if msg.is_method(&Method::Ack) => {
                self.set_state(State::Confirmed);
                if let Some(sender) = self.tx.lock().unwrap().take() {
                    sender.send(()).unwrap();
                }
                self.terminate();
            }
            State::Proceeding => {
                self.retransmit().await?;
            }
            _ => (),
        }

        Ok(())
    }

    pub async fn respond<'a>(&self, response: &mut OutgoingResponse<'a>) -> Result<()> {
        if response.is_provisional() {
            self.set_state(State::Proceeding);
            self.tsx_send_msg(response).await?;
            return Ok(());
        }
        let state = self.get_state();
        self.tsx_send_msg(response).await?;

        if matches!(state, State::Completed | State::Terminated) {
            return Ok(());
        }

        match response.status_code().into_i32() {
            200..=299 => {
                self.set_state(State::Terminated);
                self.on_terminated();
            }
            300..=699 => {
                self.set_state(State::Completed);
                let (tx, rx) = oneshot::channel();
                self.tx.lock().unwrap().replace(tx);
                self.start_retrans_timer(rx);
            }
            _ => unreachable!(),
        }
        Ok(())
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
        &self.tsx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        endpoint::Builder,
        message::StatusCode,
        transaction::{mock, TransactionLayer},
    };
    use tokio::time::Duration;

    async fn tsx_uas_params<'a>() -> (Endpoint, IncomingRequest<'a>) {
        let endpoint = Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build();
        let request = mock::request(Method::Invite);

        (endpoint.await, request)
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_180_ringing() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 100);

        let response = &mut mock::response(StatusCode::Ringing);
        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 180);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_ack_and_terminates() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::new(&endpoint, &mut request);

        let response = &mut mock::response(StatusCode::Ok);
        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 200);

        let request = &mock::request(Method::Ack);
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.get_state() == State::Terminated);
    }

    #[tokio::test]
    async fn test_invite_retransmit_100_trying() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::new(&endpoint, &mut request);

        let response = &mut mock::response(StatusCode::Trying);
        tsx.respond(response).await.unwrap();

        let request = &mock::request(Method::Invite);
        tsx.recv_msg(request).await.unwrap();

        let response = &mut mock::response(StatusCode::Ok);
        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 200);
        assert!(tsx.retrans_count() == 1);
        assert!(tsx.get_state() == State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_invite_timer_g_retransmission() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::new(&endpoint, &mut request);

        let response = &mut mock::response(StatusCode::BusyHere);
        tsx.respond(response).await.unwrap();
        assert!(tsx.get_state() == State::Completed);

        time::sleep(TsxUasInv::T1 + Duration::from_millis(1)).await;
        assert!(tsx.retrans_count() == 1);

        time::sleep(TsxUasInv::T1 * 2 + Duration::from_millis(1)).await;
        assert!(tsx.retrans_count() == 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_h_expiration() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::new(&endpoint, &mut request);

        let response = &mut mock::response(StatusCode::BusyHere);

        tsx.respond(response).await.unwrap();
        assert!(tsx.get_state() == State::Completed);

        time::sleep(TsxUasInv::T1 * 64 + Duration::from_millis(1)).await;
        assert!(tsx.get_state() == State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_ack_received_before_timer_h() {
        let (endpoint, mut request) = tsx_uas_params().await;
        let tsx = TsxUasInv::new(&endpoint, &mut request);

        let response = &mut mock::response(StatusCode::BusyHere);
        tsx.respond(response).await.unwrap();
        assert!(tsx.get_state() == State::Completed);

        let request = &mock::request(Method::Ack);
        tsx.recv_msg(request).await.unwrap();
        assert!(tsx.get_state() == State::Confirmed);

        time::sleep(TsxUasInv::T4 + Duration::from_millis(1)).await;

        assert!(tsx.get_state() == State::Terminated);
    }
}
