use crate::{
    endpoint::Endpoint,
    message::SipMethod,
    transaction::{SipTransaction, State, Transaction, T1, T2, T4},
    transport::{IncomingMessage, IncomingRequest, OutgoingMessage},
};
use async_trait::async_trait;
use std::{
    cmp, io,
    ops::Deref,
};
use tokio::{
    pin,
    sync::oneshot,
    time::{self, Instant},
};

pub struct UasInvTsx {
    tsx: Transaction,
    tx_state_confirmed: Option<oneshot::Sender<()>>,
}

impl UasInvTsx {
    pub fn new(
        endpoint: &Endpoint,
        request: &IncomingRequest,
    ) -> Self {
        assert!(matches!(request.method(), SipMethod::Invite));

        Self {
            tsx: (request, endpoint).into(),
            tx_state_confirmed: None,
        }
    }
}

//The TU passes any number of provisional responses to the server
// transaction.
#[async_trait]
impl SipTransaction for UasInvTsx {
    async fn recv_msg(
        &mut self,
        msg: IncomingMessage,
    ) -> io::Result<()> {
        let IncomingMessage::Request(request) = msg else {
            return Ok(());
        };
        let state = self.get_state();
        if request.is_method(&SipMethod::Ack)
            && state == State::Completed
        {
            self.set_state(State::Confirmed);
            if let Some(sender) = self.tx_state_confirmed.take() {
                sender.send(()).unwrap();
            }
            self.terminate();
            return Ok(());
        }

        if matches!(state, State::Proceeding) {
            self.retransmit().await?;
        }
        
        Ok(())
    }

    async fn send_msg(
        &mut self,
        msg: OutgoingMessage,
    ) -> io::Result<()> {
        let OutgoingMessage::Response(response) = msg else {
            return Ok(());
        };
        let code = response.status_code().into_u32();
        if response.is_provisional() {
            self.set_state(State::Proceeding);
            self.send(response).await?;
            return Ok(());
        }

        self.send(response).await?;
        if let State::Completed = self.get_state() {
            return Ok(());
        }

        match code {
            200..=299 => {
                self.set_state(State::Terminated);
            }
            300..=699 => {
                self.set_state(State::Completed);

                let tsx = self.clone();
                let redable = self.reliable();
                let buf = self.get_last_msg_buf().unwrap();
                let (tx, mut rx) = oneshot::channel();

                self.tx_state_confirmed = Some(tx);

                tokio::spawn(async move {
                    pin! {
                        let timer_g = time::sleep(T1);
                        let timer_h = time::sleep(64*T1);
                    }
                    loop {
                        tokio::select! {
                            _ = &mut timer_g => {
                                if !redable && tsx.get_state() != State::Confirmed {
                                    let _ = tsx.transport.send(&buf, &tsx.addr).await;
                                    let retransmissions = tsx.increment_retransmission_count();
                                    let retransmissions = 2u32.pow(retransmissions + 1);
                                    let next_interval = cmp::min(T1*retransmissions, T2);
                                    timer_g.as_mut().reset(Instant::now() + next_interval);
                                }
                            }
                            _= &mut timer_h => {
                                println!("Timer H Expired!");
                                tsx.set_state(State::Terminated);
                                tsx.on_terminated();
                                break;
                            }
                            _ = &mut rx => {
                                println!("Got confirmed state!");
                                break;
                            }
                        }
                    }
                });
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn terminate(&mut self) {
        if self.reliable() {
            self.on_terminated();
            return;
        }
        let tsx = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(T4).await;
            tsx.on_terminated();
        });
    }
}

impl Deref for UasInvTsx {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.tsx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{endpoint::EndpointBuilder, message::StatusCode, transaction::mock};
    use tokio::time::Duration;

    fn tsx_uas_params() -> (Endpoint, IncomingMessage) {
        let endpoint = EndpointBuilder::new().build();
        let request = mock::request(SipMethod::Invite);

        (endpoint, request)
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let (endpoint, request) = tsx_uas_params();
        let request = request.request().unwrap();
        let mut tsx = UasInvTsx::new(&endpoint, request);
        let response = mock::response(StatusCode::Trying);

        tsx.send_msg(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_u32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_180_ringing() {
        let (endpoint, request) = tsx_uas_params();
        let request = request.request().unwrap();
        let mut tsx = UasInvTsx::new(&endpoint, request);
        let response = mock::response(StatusCode::Trying);

        tsx.send_msg(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_u32() == 100);

        let response = mock::response(StatusCode::Ringing);
        tsx.send_msg(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_u32() == 180);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_ack_and_terminates() {
        let (endpoint, request) = tsx_uas_params();
        let request = request.request().unwrap();
        let mut tsx = UasInvTsx::new(&endpoint, request);

        let response = mock::response(StatusCode::Ok);
        tsx.send_msg(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_u32() == 200);

        let request = mock::request(SipMethod::Ack);
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.get_state() == State::Terminated);
    }

    #[tokio::test]
    async fn test_invite_retransmit_100_trying() {
        let (endpoint, request) = tsx_uas_params();
        let request = request.request().unwrap();
        let mut tsx = UasInvTsx::new(&endpoint, request);

        let response = mock::response(StatusCode::Trying);
        tsx.send_msg(response).await.unwrap();

        let request = mock::request(SipMethod::Invite);
        tsx.recv_msg(request).await.unwrap();

        let response = mock::response(StatusCode::Ok);
        tsx.send_msg(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_u32() == 200);
        assert!(tsx.retransmission_count() == 1);
        assert!(tsx.get_state() == State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_invite_timer_g_retransmission() {
        let (endpoint, request) = tsx_uas_params();
        let request = request.request().unwrap();
        let mut tsx = UasInvTsx::new(&endpoint, request);

        let response = mock::response(StatusCode::BusyHere);
        tsx.send_msg(response).await.unwrap();
        assert!(tsx.get_state() == State::Completed);

        time::sleep(T1 + Duration::from_millis(1)).await;
        assert!(tsx.retransmission_count() == 1);

        time::sleep(T1 * 2 + Duration::from_millis(1)).await;
        assert!(tsx.retransmission_count() == 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_h_expiration() {
        let (endpoint, request) = tsx_uas_params();
        let request = request.request().unwrap();
        let mut tsx = UasInvTsx::new(&endpoint, request);

        let response = mock::response(StatusCode::BusyHere);
        tsx.send_msg(response).await.unwrap();
        assert!(tsx.get_state() == State::Completed);

        time::sleep(T1 * 64 + Duration::from_millis(1)).await;
        assert!(tsx.get_state() == State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_ack_received_before_timer_h() {
        let (endpoint, request) = tsx_uas_params();
        let request = request.request().unwrap();
        let mut tsx = UasInvTsx::new(&endpoint, request);

        let response = mock::response(StatusCode::BusyHere);
        tsx.send_msg(response).await.unwrap();
        assert!(tsx.get_state() == State::Completed);

        let request = mock::request(SipMethod::Ack);
        tsx.recv_msg(request.into()).await.unwrap();
        assert!(tsx.get_state() == State::Confirmed);

        tokio::time::sleep(T4 + Duration::from_millis(1)).await;

        assert!(tsx.get_state() == State::Terminated);
    }
}
