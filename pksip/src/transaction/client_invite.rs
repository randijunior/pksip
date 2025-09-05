use std::cmp;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;

use futures_util::future::{
    Either, {self},
};
use tokio::pin;
use tokio::time::{self};

use crate::header::{CSeq, Header, Headers};
use crate::message::{Request, RequestLine, SipMethod};
use crate::transaction::{State, Transaction, T1, T2};
use crate::transport::{IncomingResponse, OutgoingRequest};
use crate::{Result, SipEndpoint};

/// Represents a Client INVITE transaction.
#[derive(Clone)]
pub struct ClientInvTransaction {
    transaction: Transaction,
    request: Arc<OutgoingRequest>,
}

const TIMER_D: Duration = Duration::from_secs(32);

impl ClientInvTransaction {
    /// TODO: doc
    pub async fn send(mut request: OutgoingRequest, endpoint: &SipEndpoint) -> Result<()> {
        let transactions = endpoint.transactions();
        let method = request.msg.method();

        assert!(
            matches!(method, SipMethod::Invite),
            "Invalid method for client INVITE transaction: expected INVITE, got: {}",
            method
        );

        let transaction = Transaction::new_uac_inv(&request, endpoint);
        transaction.tsx_send_request(&mut request).await?;
        let request = Arc::new(request);

        let uac_inv = Self {
            transaction,
            request,
        };

        transactions.add_client_inv_tsx_to_map(uac_inv.clone());

        tokio::spawn(uac_inv.retrans_loop());

        Ok(())
    }

    async fn retrans_loop(self) -> Result<()> {
        pin! {
            let timer_b = time::sleep(64 * T1);
            let timer_a = if !self.reliable() {
                Either::Left(time::sleep(T1))
            } else {
                Either::Right(future::pending::<()>())
            };
        }

        'retrans: loop {
            tokio::select! {
                _ = &mut timer_a, if self.is_calling() => {
                    match self.retransmit().await {
                        Ok(retrans) =>  {
                            let retrans = T1 * (1 << retrans);
                            let interval = cmp::min(retrans, T2);
                            let sleep = time::sleep(interval);
                            timer_a.set(Either::Left(sleep));
                        },
                        Err(err) =>  {
                            log::info!("Failed to retransmit: {}", err);
                        },
                    }
                }
                _ = &mut timer_b, if self.is_calling() => {
                    // Timeout
                    self.on_terminated();
                    break 'retrans Ok(());
                }
            }
        }
    }

    pub(crate) async fn receive(&self, response: &IncomingResponse) -> Result<bool> {
        let code = response.response.code();
        self.set_last_status_code(code);

        match self.get_state() {
            State::Calling if code.is_provisional() => {
                self.change_state_to(State::Proceeding);
            }
            State::Calling | State::Proceeding if matches!(code.as_u16(), 300..=699) => {
                self.change_state_to(State::Completed);
                let mut ack = self.create_ack(response);

                self.tsx_send_request(&mut ack).await?;
                self.terminate();
            }
            State::Calling | State::Proceeding if code.is_final() => {
                self.on_terminated();
            }
            State::Completed => {
                // 17.1.1.2 INVITE Client Transaction
                // Any retransmissions of the final response that are
                // received while in the "Completed" state
                // MUST cause the ACK to be re-passed to the
                // transport layer for retransmission, but the newly
                // received response MUST NOT be passed up
                // to the TU.
                self.retransmit().await?;

                return Ok(true);
            }
            _ => (),
        }
        Ok(false)
    }

    fn create_ack<'a>(&self, response: &IncomingResponse) -> OutgoingRequest {
        let mut via = None;
        let mut cseq = None;
        let mut call_id = None;
        let mut from = None;

        for header in self.request.msg.headers.iter() {
            match header {
                Header::From(f) => from = Some(f),
                Header::Via(v) => via = Some(v),
                Header::CallId(c) => call_id = Some(c),
                Header::CSeq(c) => cseq = Some(*c),
                _ => continue,
            }
        }
        let via = via.unwrap().clone();
        let cseq = cseq.unwrap();
        let call_id = call_id.unwrap().clone();
        let from = from.unwrap().clone();

        let mut iter = response
            .response
            .headers
            .iter()
            .filter_map(|header| match header {
                Header::To(to_hdr) => Some(to_hdr),
                _ => None,
            });

        let to = iter.next().unwrap();
        let cseq = CSeq {
            method: SipMethod::Ack,
            ..cseq
        };

        let mut ack_hdrs = Headers::with_capacity(5);

        let via = via.clone();
        let from = from.clone();
        let to = to.clone();
        let cid = call_id.clone();

        let uri = self.request.msg.req_line.uri.clone();

        ack_hdrs.push(Header::Via(via));
        ack_hdrs.push(Header::From(from));
        ack_hdrs.push(Header::To(to));
        ack_hdrs.push(Header::CallId(cid));
        ack_hdrs.push(Header::CSeq(cseq));

        OutgoingRequest {
            msg: Request {
                req_line: RequestLine {
                    method: SipMethod::Ack,
                    uri,
                },
                headers: ack_hdrs,
                body: None,
            },
            addr: self.addr(),
            buf: None,
            transport: self.transport().clone(),
        }
    }

    pub(super) fn terminate(&self) {
        if self.reliable() {
            self.on_terminated();
        } else {
            // Start timer D
            self.schedule_termination(TIMER_D);
        }
    }
}

impl DerefMut for ClientInvTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for ClientInvTransaction {
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
    use crate::transaction::mock;
    /*

    #[tokio::test]
    async fn test_state_calling() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);

        let uac_inv = ClientInvTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac_inv.get_state(), State::Calling);
    }

    #[tokio::test]
    async fn test_state_proceeding() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::Trying);

        let uac_inv = ClientInvTransaction::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.get_state(), State::Proceeding);
    }

    #[tokio::test]
    async fn test_state_completed() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::BusyHere);

        let uac_inv = ClientInvTransaction::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.last_status_code(), Some(StatusCode::BusyHere));
        assert_eq!(uac_inv.get_state(), State::Completed);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_a() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);

        let uac_inv = ClientInvTransaction::send(request, &endpoint).await.unwrap();

        assert!(uac_inv.retrans_count() == 0);
        assert_eq!(uac_inv.get_state(), State::Calling);

        time::sleep(Duration::from_millis(500 + 1)).await;
        assert!(uac_inv.retrans_count() == 1);

        time::sleep(Duration::from_secs(1) + Duration::from_millis(1)).await;
        assert!(uac_inv.retrans_count() == 2);

        time::sleep(Duration::from_secs(2) + Duration::from_millis(1)).await;
        assert!(uac_inv.retrans_count() == 3);

        time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
        assert!(uac_inv.retrans_count() == 4);

        time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
        assert!(uac_inv.retrans_count() == 5);

        time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
        assert!(uac_inv.retrans_count() == 6);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_b() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);

        let uac_inv = ClientInvTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac_inv.get_state(), State::Calling);

        time::sleep(ClientInvTransaction::T1 * 64 + Duration::from_millis(1)).await;

        assert!(uac_inv.get_state() == State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_d() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::BusyHere);

        let uac_inv = ClientInvTransaction::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.get_state(), State::Completed);

        time::sleep(TIMER_D + Duration::from_millis(1)).await;

        assert!(uac_inv.get_state() == State::Terminated);
    }
    */
}
