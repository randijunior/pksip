use async_trait::async_trait;

use crate::{transaction::TsxState, transport::Transport};
use std::{
    io,
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

use super::{SipTransaction, Transaction, TsxMsg, TsxStateMachine, T1};

pub struct ServerNonInviteTsx(Transaction);

impl ServerNonInviteTsx {
    // The state machine is initialized in the "Trying" state and is passed
    // a request other than INVITE or ACK when initialized.
    pub fn new(addr: SocketAddr, transport: Transport) -> Self {
        Self(Transaction {
            state: TsxStateMachine::new(TsxState::Trying),
            addr,
            transport,
            last_response: None,
            tx: None,
        })
    }
}

#[async_trait]
impl SipTransaction for ServerNonInviteTsx {
    async fn recv_msg(&mut self, msg: TsxMsg) -> io::Result<()> {
        let state = self.get_state();
        if let TsxState::Completed = state {
            return Ok(());
        }
        let TsxMsg::Response(response) = msg else {
            if let TsxState::Trying = state {
                // Once in the "Trying" state, any further request
                // retransmissions are discarded.
                return Ok(());
            }
            if let TsxState::Proceeding = state {
                self.retransmit().await?;
            }
            return Ok(());
        };

        if response.is_provisional() {
            self.send(response).await?;
            if let TsxState::Trying = state {
                self.state.proceeding();
                return Ok(());
            }
        } else {
            self.send(response).await?;
            if matches!(state, TsxState::Proceeding | TsxState::Trying) {
                self.state.completed();
                self.do_terminate(T1 * 64);
            }
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
        headers::{CSeq, CallId, Headers},
        message::{SipMethod, SipResponse, StatusCode},
        transport::{
            udp::mock::MockUdpTransport, OutgoingInfo, OutgoingResponse,
            RequestHeaders, Transport,
        },
    };

    fn resp(c: StatusCode) -> TsxMsg {
        let from = "sip:alice@127.0.0.1:5060".parse().unwrap();
        let to = "sip:bob@127.0.0.1:5060".parse().unwrap();
        let cseq = CSeq {
            cseq: 1,
            method: SipMethod::Options,
        };
        let callid = CallId::new("bs9ki9iqbee8k5kal8mpqb");
        let hdrs = RequestHeaders {
            via: vec![],
            from,
            to,
            callid,
            cseq,
        };
        let transport = Transport::new(MockUdpTransport);
        let info = OutgoingInfo {
            addr: transport.addr(),
            transport,
        };
        let msg = SipResponse::new(c.into(), Headers::new(), None);
        let response = OutgoingResponse {
            hdrs,
            msg,
            info,
            buf: None,
        };

        response.into()
    }

    #[tokio::test]
    async fn test_receives_100_trying() {
        let tp = Transport::new(MockUdpTransport);
        let mut tsx = ServerNonInviteTsx::new(tp.addr(), tp);

        tsx.recv_msg(resp(StatusCode::Trying)).await.unwrap();

        assert!(tsx.last_response_code() == Some(100));
        assert!(tsx.state.is_proceeding());
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let tp = Transport::new(MockUdpTransport);
        let mut tsx = ServerNonInviteTsx::new(tp.addr(), tp);

        tsx.recv_msg(resp(StatusCode::Ok)).await.unwrap();

        assert!(tsx.last_response_code() == Some(200));
        assert!(tsx.state.is_completed());
    }
}
