use super::{
    SipTransaction, Transaction, TsxMsg, TsxState, TsxStateMachine, T1, T4,
};
use crate::{
    endpoint::Endpoint,
    message::{SipMethod, StatusCode},
    transaction::T2,
    transport::IncomingRequest,
};
use async_trait::async_trait;
use std::{
    cmp, io,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::{
    pin,
    sync::oneshot,
    time::{self, Instant},
};

pub struct TsxUasInv {
    tsx: Transaction,
    tx_confirmed_state: Option<oneshot::Sender<()>>,
}

impl TsxUasInv {
    pub async fn new(
        request: &mut IncomingRequest,
        endpoint: &Endpoint,
    ) -> io::Result<Self> {
        let mut tsx = Self {
            tsx: Transaction {
                state: TsxStateMachine::new(TsxState::Proceeding),
                addr: request.packet().addr,
                transport: request.transport().clone(),
                last_response: None,
                tx: None,
                retransmit_count: AtomicUsize::new(0).into(),
            },
            tx_confirmed_state: None,
        };
        let response = endpoint
            .new_response(request, StatusCode::Trying.into())
            .await?;
        tsx.recv_msg(response.into()).await?;

        Ok(tsx)
    }
}

//The TU passes any number of provisional responses to the server
// transaction.
#[async_trait]
impl SipTransaction for TsxUasInv {
    async fn recv_msg(&mut self, msg: TsxMsg) -> io::Result<()> {
        let state = self.get_state();
        match msg {
            TsxMsg::UasRequest(request) => {
                if request.is_method(&SipMethod::Ack) && state.is_completed() {
                    self.state.confirmed();
                    if let Some(sender) = self.tx_confirmed_state.take() {
                        sender.send(()).unwrap();
                    }
                    self.do_terminate(T4);
                    return Ok(());
                }
                /*
                 * If a request retransmission is received while in the
                 * "Proceeding" state, the most recent provisional response
                 * that was received from the TU MUST be passed
                 * to the transport layer for retransmission.
                 */
                if matches!(state, TsxState::Proceeding) {
                    self.retransmit().await?;
                }
                return Ok(());
            }
            TsxMsg::UasResponse(response) => {
                let code = response.status_code().into_u32();
                if response.is_provisional() {
                    self.send(response).await?;
                    return Ok(());
                }
                // If, while in the "Proceeding" state, the TU passes a 2xx response to
                // the server transaction, the server transaction MUST pass this
                // response to the transport layer for transmission.
                // The server transaction MUST then transition to the "Terminated" state.
                if let TsxState::Proceeding = state {
                    self.send(response).await?;
                    match code {
                        200..=299 => {
                            self.state.terminated();
                        }
                        300..=699 => {
                            self.state.completed();
                            let buf = self
                                .last_response
                                .as_ref()
                                .unwrap()
                                .buf
                                .as_ref()
                                .unwrap()
                                .clone();
                            let transport = self.transport.clone();
                            let addr = self.addr;
                            let redable = self.reliable();
                            let sender = self.tx.take();
                            let state = self.state.clone();
                            let retrans_count = self.retransmit_count.clone();
                            let (tx_confirmed_state, mut rx_confirmed_state) =
                                oneshot::channel();
                            self.tx_confirmed_state = Some(tx_confirmed_state);

                            tokio::spawn(async move {
                                pin! {
                                    let timer_g = time::sleep(T1);
                                    let timer_h = time::sleep(64*T1);
                                }
                                loop {
                                    tokio::select! {
                                        _ = &mut timer_g => {
                                            if !redable && !state.is_confirmed() {
                                                let _ = transport.send(&buf, addr).await;
                                                let retransmissions = retrans_count.fetch_add(1, Ordering::SeqCst);
                                                let retransmissions = 2u32.pow((retransmissions + 1) as u32);
                                                let next_interval = cmp::min(T1*retransmissions, T2);
                                                timer_g.as_mut().reset(Instant::now() + next_interval);
                                            }
                                        }
                                        _= &mut timer_h => {
                                            println!("Timer H Expired!");
                                            state.terminated();
                                            if let Some(sender) = sender {
                                                sender.send(()).unwrap();
                                            }
                                            break;
                                        }
                                        _ = &mut rx_confirmed_state => {
                                            println!("Got confirmed state!");
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

impl Deref for TsxUasInv {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.tsx
    }
}

impl DerefMut for TsxUasInv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tsx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{endpoint::EndpointBuilder, transaction::mock};
    use tokio::time::Duration;

    fn tsx_uas_params() -> (Endpoint, TsxMsg) {
        let endpoint = EndpointBuilder::new().build();
        let request = mock::request(SipMethod::Invite);

        (endpoint, request)
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let (endpoint, mut request) = tsx_uas_params();
        let incoming = request.uas_request_mut().unwrap();

        let tsx = TsxUasInv::new(incoming, &endpoint).await.unwrap();

        assert!(tsx.last_response_code().unwrap().into_u32() == 100);
        assert!(tsx.state.is_proceeding());
    }

    #[tokio::test]
    async fn test_receives_180_ringing() {
        let (endpoint, mut request) = tsx_uas_params();
        let incoming = request.uas_request_mut().unwrap();

        let mut tsx = TsxUasInv::new(incoming, &endpoint).await.unwrap();

        assert!(tsx.last_response_code().unwrap().into_u32() == 100);

        let response = mock::response(StatusCode::Ringing);
        tsx.recv_msg(response.into()).await.unwrap();

        assert!(tsx.last_response_code().unwrap().into_u32() == 180);
        assert!(tsx.state.is_proceeding());
    }

    #[tokio::test]
    async fn test_receives_ack_and_terminates() {
        let (endpoint, mut request) = tsx_uas_params();
        let incoming = request.uas_request_mut().unwrap();

        let mut tsx = TsxUasInv::new(incoming, &endpoint).await.unwrap();

        let response = mock::response(StatusCode::Ok);
        tsx.recv_msg(response.into()).await.unwrap();

        assert!(tsx.last_response_code().unwrap().into_u32() == 200);

        let request = mock::request(SipMethod::Ack);
        tsx.recv_msg(request.into()).await.unwrap();

        assert!(tsx.state.is_terminated());
    }

    #[tokio::test]
    async fn test_invite_retransmit_100_trying() {
        let (endpoint, mut request) = tsx_uas_params();
        let incoming = request.uas_request_mut().unwrap();

        let mut tsx = TsxUasInv::new(incoming, &endpoint).await.unwrap();

        let request = mock::request(SipMethod::Invite);
        tsx.recv_msg(request.into()).await.unwrap();

        let response = mock::response(StatusCode::Ok);
        tsx.recv_msg(response.into()).await.unwrap();

        assert!(tsx.last_response_code().unwrap().into_u32() == 200);
        assert!(tsx.retransmission_count() == 1);
        assert!(tsx.state.is_terminated());
    }

    #[tokio::test(start_paused = true)]
    async fn test_invite_timer_g_retransmission() {
        let (endpoint, mut request) = tsx_uas_params();
        let incoming = request.uas_request_mut().unwrap();

        let mut tsx = TsxUasInv::new(incoming, &endpoint).await.unwrap();

        let response = mock::response(StatusCode::BusyHere);
        tsx.recv_msg(response.into()).await.unwrap();
        assert!(tsx.state.is_completed());

        time::sleep(T1 + Duration::from_millis(1)).await;
        assert!(tsx.retransmission_count() == 1);

        time::sleep(T1 * 2 + Duration::from_millis(1)).await;
        assert!(tsx.retransmission_count() == 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_h_expiration() {
        let (endpoint, mut request) = tsx_uas_params();
        let incoming = request.uas_request_mut().unwrap();

        let mut tsx = TsxUasInv::new(incoming, &endpoint).await.unwrap();

        let response = mock::response(StatusCode::BusyHere);
        tsx.recv_msg(response.into()).await.unwrap();
        assert!(tsx.state.is_completed());

        time::sleep(T1 * 64 + Duration::from_millis(1)).await;
        assert!(tsx.state.is_terminated());
    }

    #[tokio::test(start_paused = true)]
    async fn test_ack_received_before_timer_h() {
        let (endpoint, mut request) = tsx_uas_params();
        let incoming = request.uas_request_mut().unwrap();

        let mut tsx = TsxUasInv::new(incoming, &endpoint).await.unwrap();

        let response = mock::response(StatusCode::BusyHere);
        tsx.recv_msg(response.into()).await.unwrap();
        assert!(tsx.state.is_completed());

        let request = mock::request(SipMethod::Ack);
        tsx.recv_msg(request.into()).await.unwrap();
        assert!(tsx.state.is_confirmed());

        tokio::time::sleep(T4 + Duration::from_millis(1)).await;

        assert!(tsx.state.is_terminated());
    }
}
