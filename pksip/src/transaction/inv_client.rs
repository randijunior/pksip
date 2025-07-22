use std::{
    cmp,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use futures_util::future::{self, Either};
use tokio::{
    pin,
    time::{self},
};

use crate::{
    headers::{self, CSeq, Header, Headers},
    message::{Request, RequestLine, SipMethod, Uri},
    transaction::{Transaction, State},
    transport::{IncomingResponse, OutgoingRequest, RequestHeaders},
    Endpoint, Result,
};

use super::TransactionInner;

struct OriginalRequest {
    uri: Uri<'static>,
    via: headers::Via<'static>,
    from: headers::From<'static>,
    cseq: CSeq,
    call_id: headers::CallId<'static>,
}

/// Represents a Client INVITE transaction.
#[derive(Clone)]
pub struct InvClientTransaction {
    transaction: TransactionInner,
    request: Arc<OriginalRequest>,
}

const TIMER_D: Duration = Duration::from_secs(32);

impl InvClientTransaction {
    pub async fn send(mut request: OutgoingRequest<'_>, endpoint: &Endpoint) -> Result<InvClientTransaction> {
        let tsx_layer = endpoint.get_tsx_layer();
        let method = request.msg.method();

        assert!(
            matches!(method, SipMethod::Invite),
            "Invalid method for client INVITE transaction: expected INVITE, got: {}",
            method
        );

        let transaction = TransactionInner::create_uac_inv(&request, endpoint);
        transaction.tsx_send_request(&mut request).await?;

        let mut via = None;
        let mut cseq = None;
        let mut call_id = None;
        let mut from = None;

        for header in request.msg.headers.iter() {
            match header {
                Header::From(f) => from = Some(f),
                Header::Via(v) => via = Some(v),
                Header::CallId(c) => call_id = Some(c),
                Header::CSeq(c) => cseq = Some(*c),
                _ => continue,
            }
        }

        let via = via.unwrap().clone().into_owned();
        let cseq = cseq.unwrap();
        let call_id = call_id.unwrap().clone().into_owned();
        let from = from.unwrap().clone().into_owned();

        let uri = request.msg.req_line.uri.into_owned();

        let request = Arc::new(OriginalRequest {
            uri,
            via,
            cseq,
            call_id,
            from,
        });
        let uac_inv = InvClientTransaction { transaction, request };

        tsx_layer.add_client_inv_tsx_to_map(uac_inv.clone());

        tokio::spawn(uac_inv.clone().tsx_retrans_task());

        Ok(uac_inv)
    }

    async fn tsx_retrans_task(self) -> Result<()> {
        pin! {
            let timer_b = time::sleep(64 * Self::T1);
            let timer_a = if !self.reliable() {
                Either::Left(time::sleep(Self::T1))
            } else {
                Either::Right(future::pending::<()>())
            };
        }

        'retrans: loop {
            tokio::select! {
                _ = &mut timer_a, if self.is_calling() => {
                    match self.retransmit().await {
                        Ok(retrans) =>  {
                            let retrans = Self::T1 * (1 << retrans);
                            let interval = cmp::min(retrans, Self::T2);
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

    pub(crate) async fn receive(&self, response: &IncomingResponse<'_>) -> Result<bool> {
        let code = response.response.code();
        self.set_last_status_code(code);

        match self.get_state() {
            State::Calling if code.is_provisional() => {
                self.change_state_to(State::Proceeding);
            }
            State::Calling | State::Proceeding if matches!(code.into_i32(), 300..=699) => {
                self.change_state_to(State::Completed);
                let mut ack = self.create_ack(response);

                self.tsx_send_request(&mut ack).await?;
                self.terminate();
            }
            State::Calling | State::Proceeding if code.is_final() => {
                self.on_terminated();
            }
            State::Completed => {
                // 17.1.1.2 INVITE Client TransactionInner
                // Any retransmissions of the final response that are received while in
                // the "Completed" state MUST cause the ACK to be re-passed to the
                // transport layer for retransmission, but the newly received response
                // MUST NOT be passed up to the TU.
                self.retransmit().await?;

                return Ok(true);
            }
            _ => (),
        }
        Ok(false)
    }

    fn create_ack<'a>(&self, response: &'a IncomingResponse<'a>) -> OutgoingRequest<'a> {
        let mut iter = response.response.headers.iter().filter_map(|header| match header {
            Header::To(to_hdr) => Some(to_hdr),
            _ => None,
        });

        let to = iter.next().unwrap();
        let cseq = CSeq {
            method: SipMethod::Ack,
            ..self.request.cseq
        };

        let headers = &self.request;
        let mut ack_hdrs = Headers::with_capacity(5);

        let via = headers.via.clone();
        let from = headers.from.clone();
        let to = to.clone();
        let cid = headers.call_id.clone();

        ack_hdrs.push(Header::Via(via));
        ack_hdrs.push(Header::From(from));
        ack_hdrs.push(Header::To(to));
        ack_hdrs.push(Header::CallId(cid));
        ack_hdrs.push(Header::CSeq(cseq));

        OutgoingRequest {
            msg: Request {
                req_line: RequestLine {
                    method: SipMethod::Ack,
                    uri: self.request.uri.clone(),
                },
                headers: ack_hdrs,
                body: None,
            },
            addr: self.addr(),
            buf: None,
            transport: self.transport().clone(),
        }
    }
}

#[async_trait::async_trait]
impl Transaction for InvClientTransaction {
    fn terminate(&self) {
        if self.reliable() {
            self.on_terminated();
        } else {
            // Start timer D
            self.schedule_termination(TIMER_D);
        }
    }
}

impl DerefMut for InvClientTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for InvClientTransaction {
    type Target = TransactionInner;

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
    async fn test_state_calling() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);

        let uac_inv = InvClientTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac_inv.get_state(), State::Calling);
    }

    #[tokio::test]
    async fn test_state_proceeding() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::Trying);

        let uac_inv = InvClientTransaction::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.get_state(), State::Proceeding);
    }

    #[tokio::test]
    async fn test_state_completed() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::BusyHere);

        let uac_inv = InvClientTransaction::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.last_status_code(), Some(StatusCode::BusyHere));
        assert_eq!(uac_inv.get_state(), State::Completed);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_a() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);

        let uac_inv = InvClientTransaction::send(request, &endpoint).await.unwrap();

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

        let uac_inv = InvClientTransaction::send(request, &endpoint).await.unwrap();

        assert_eq!(uac_inv.get_state(), State::Calling);

        time::sleep(InvClientTransaction::T1 * 64 + Duration::from_millis(1)).await;

        assert!(uac_inv.get_state() == State::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_d() {
        let endpoint = mock::default_endpoint().await;
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::BusyHere);

        let uac_inv = InvClientTransaction::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.get_state(), State::Completed);

        time::sleep(TIMER_D + Duration::from_millis(1)).await;

        assert!(uac_inv.get_state() == State::Terminated);
    }
}
