use async_trait::async_trait;

use crate::{transaction::TsxState, transport::IncomingRequest};
use std::{
    io,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicUsize,
};

use super::{SipTransaction, Transaction, TsxMsg, TsxStateMachine, T1};

pub struct TsxUas(Transaction);

impl TsxUas {
    // The state machine is initialized in the "Trying" state and is passed
    // a request other than INVITE or ACK when initialized.
    pub fn new(request: &IncomingRequest) -> Self {
        Self(Transaction {
            state: TsxStateMachine::new(TsxState::Trying),
            addr: request.packet().addr,
            transport: request.transport().clone(),
            last_response: None,
            tx: None,
            retransmit_count: AtomicUsize::new(0).into(),
        })
    }
}

#[async_trait]
impl SipTransaction for TsxUas {
    async fn recv_msg(&mut self, msg: TsxMsg) -> io::Result<()> {
        let state = self.get_state();
        let TsxMsg::UasResponse(response) = msg else {
            if state == TsxState::Trying {
                // Once in the "Trying" state, any further request
                // retransmissions are discarded.
                return Ok(());
            }
            if matches!(state, TsxState::Proceeding | TsxState::Completed) {
                self.retransmit().await?;
            }
            return Ok(());
        };

        let provisional = response.is_provisional();
        self.send(response).await?;

        if state == TsxState::Completed {
            return Ok(());
        }

        if provisional && state == TsxState::Trying {
            self.state.proceeding();
            return Ok(());
        }
        if matches!(state, TsxState::Trying | TsxState::Proceeding) {
            self.state.completed();
            self.do_terminate(T1 * 64);
        }
        Ok(())
    }
}

impl Deref for TsxUas {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TsxUas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{self, Duration};

    use super::*;
    use crate::{
        message::{SipMethod, StatusCode},
        transaction::mock,
    };

    #[tokio::test]
    async fn test_receives_100_trying() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.uas_request().unwrap();
        let mut tsx = TsxUas::new(incoming);
        let response = mock::response(StatusCode::Trying);

        tsx.recv_msg(response).await.unwrap();

        assert!(tsx.last_response_code().unwrap().into_u32() == 100);
        assert!(tsx.state.is_proceeding());
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.uas_request().unwrap();
        let mut tsx = TsxUas::new(incoming);
        let response = mock::response(StatusCode::Ok);

        tsx.recv_msg(response).await.unwrap();

        assert!(tsx.last_response_code().unwrap().into_u32() == 200);
        assert!(tsx.state.is_completed());
    }

    #[tokio::test]
    async fn test_retransmit_proceeding() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.uas_request().unwrap();
        let mut tsx = TsxUas::new(incoming);
        let response = mock::response(StatusCode::Trying);

        tsx.recv_msg(response).await.unwrap();
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.retransmission_count() == 1);
        assert!(tsx.last_response_code().unwrap().into_u32() == 100);
        assert!(tsx.state.is_proceeding());
    }

    #[tokio::test]
    async fn test_retransmit_completed() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.uas_request().unwrap();
        let mut tsx = TsxUas::new(incoming);
        let response = mock::response(StatusCode::Ok);

        tsx.recv_msg(response).await.unwrap();
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.retransmission_count() == 1);
        assert!(tsx.last_response_code().unwrap().into_u32() == 200);
        assert!(tsx.state.is_completed());
    }

    #[tokio::test(start_paused = true)]
    async fn test_terminated_timer_j() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.uas_request().unwrap();
        let mut tsx = TsxUas::new(incoming);
        let response = mock::response(StatusCode::Ok);

        tsx.recv_msg(response).await.unwrap();

        time::sleep(T1 * 64 + Duration::from_millis(1)).await;

        assert!(tsx.last_response_code().unwrap().into_u32() == 200);
        assert!(tsx.state.is_terminated());
    }
}
