use async_trait::async_trait;

use crate::{
    transaction::TsxState,
    transport::IncomingRequest,
};
use std::{
    io,
    ops::{Deref, DerefMut},
};

use super::{SipTransaction, Transaction, TsxMsg, TsxStateMachine, T1};

pub struct ServerNonInviteTsx(Transaction);

impl ServerNonInviteTsx {
    // The state machine is initialized in the "Trying" state and is passed
    // a request other than INVITE or ACK when initialized.
    pub fn new(request: &IncomingRequest) -> Self {
        Self(Transaction {
            state: TsxStateMachine::new(TsxState::Trying),
            addr: request.packet().addr,
            transport: request.transport().clone(),
            last_response: None,
            tx: None,
            retransmit_count: 0
        })
    }
}

#[async_trait]
impl SipTransaction for ServerNonInviteTsx {
    async fn recv_msg(&mut self, msg: TsxMsg) -> io::Result<()> {
        let state = self.get_state();
        let completed = state.is_completed();
        let trying = state.is_trying();
        let proceding = state.is_proceeding();
        let TsxMsg::Response(response) = msg else {
            if trying {
                // Once in the "Trying" state, any further request
                // retransmissions are discarded.
                return Ok(());
            }
            if proceding || completed {
                self.retransmit().await?;
            }
            return Ok(());
        };
        let provisional = response.is_provisional();
        self.send(response).await?;

        if completed {
            return Ok(());
        }

        if provisional && trying {
            self.state.proceeding();
            return Ok(());
        }
        if trying || proceding {
            self.state.completed();
            self.do_terminate(T1 * 64);
        }
        Ok(())
    }
}

impl Deref for ServerNonInviteTsx {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ServerNonInviteTsx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        message::{SipMethod, StatusCode},
        transaction::mock,
    };

    #[tokio::test]
    async fn test_receives_100_trying() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.request().unwrap();
        let mut tsx = ServerNonInviteTsx::new(incoming);
        let response = mock::response(StatusCode::Trying);

        tsx.recv_msg(response).await.unwrap();

        assert!(tsx.last_response_code() == Some(100));
        assert!(tsx.state.is_proceeding());
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.request().unwrap();
        let mut tsx = ServerNonInviteTsx::new(incoming);
        let response = mock::response(StatusCode::Ok);

        tsx.recv_msg(response).await.unwrap();

        assert!(tsx.last_response_code() == Some(200));
        assert!(tsx.state.is_completed());
    }

    #[tokio::test]
    async fn test_retransmit_proceeding() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.request().unwrap();
        let mut tsx = ServerNonInviteTsx::new(incoming);
        let response = mock::response(StatusCode::Trying);

        tsx.recv_msg(response).await.unwrap();
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.retransmit_count == 1);
        assert!(tsx.last_response_code() == Some(100));
        assert!(tsx.state.is_proceeding());
    }

    #[tokio::test]
    async fn test_retransmit_completed() {
        let request = mock::request(SipMethod::Options);
        let incoming = request.request().unwrap();
        let mut tsx = ServerNonInviteTsx::new(incoming);
        let response = mock::response(StatusCode::Ok);

        tsx.recv_msg(response).await.unwrap();
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.retransmit_count == 1);
        assert!(tsx.last_response_code() == Some(200));
        assert!(tsx.state.is_completed());
    }
}
