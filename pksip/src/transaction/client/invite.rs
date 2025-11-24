use std::{
    cmp,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use bytes::Bytes;
use futures_util::future::{
    Either, {self},
};
use tokio::{
    pin,
    time::{self},
};

use crate::{
    Endpoint, Result,
    error::Error,
    find_map_header,
    headers::{CSeq, Header, Headers, Route},
    message::{HostPort, NameAddr, Request, RequestLine, SipMethod},
    transaction::{T1, T2, sip_transaction::Transaction},
    transport::{
        IncomingResponse, OutgoingMessage, OutgoingMessageInfo, OutgoingRequest, Transport,
    },
};

const TIMER_D: Duration = Duration::from_secs(32);

/// Represents a Client INVITE transaction.
#[derive(Clone)]
pub struct ClientInviteTx {
    /// The inner transaction.
    transaction: Transaction,
    /// The SIP request sended.
    request: Arc<OutgoingRequest>,
}


// RFC 3261 - 8.1.1 Generating the Request - Endpoint
// RFC 3261 - 8.1.2 Sending the Request - ClientInviteTx
// RFC 3263 - 4.1 Selecting a Transport Protocol (UDP/TCP/TLS)
// RFC 3263 - 4.2 Determining Port and IP Address (SRV/A/AAAA)
// RFC 3261 - 17.1 Client Transaction

impl ClientInviteTx {
    pub(crate) async fn send_request(
        endpoint: &Endpoint,
        mut request: Request,
        target: Option<(Transport, SocketAddr)>,
    ) -> Result<Self> {
        // if request.method() != SipMethod::Invite {
        //     return Err(Error::InvalidMethod);
        // }

        let (selected_transport, destination) = if let Some((tr, addr)) = target {
            // Transport and address explicitly provided
            (tr, addr)
        } else {
            // ============================================================
            // RFC 3261 8.1.2 - Sending the Request
            // RFC 3261 12.2.1.1 - Generating the Request
            // ============================================================
            let topmost_route = request
                .headers
                .iter_mut()
                .position(|header| matches!(header, Header::Route(route) if !route.name_addr.uri.lr_param))
                .map(|index| request.headers.remove(index).into_route().unwrap());

            let new_request_uri = if let Some(route) = &topmost_route {
                &route.name_addr.uri
            } else {
                &request.req_line.uri
            };

            if new_request_uri != &request.req_line.uri {
                let name_addr = NameAddr::new(request.req_line.uri.clone());
                let route = Header::Route(Route {
                    name_addr,
                    param: None,
                });
                let index = request
                    .headers
                    .iter()
                    .rposition(|h| matches!(h, Header::Route(_)));
                
                if let Some(index) = index {
                    request.headers.insert(index, route);
                } else {
                    request.headers.push(route);
                }
            }

            // ============================================================
            // RFC 3263 §4.1 — Selecting a Transport Protocol
            // ============================================================
            //
            // Decide which transport to use:
            // - If URI has "transport=xxx", use it.
            // - If URI is "sips:", use TLS over TCP.
            // - Otherwise, follow SRV fallback: _sip._udp, _sip._tcp, ...
            // ============================================================
            endpoint
                .transports()
                .select_transport(endpoint, new_request_uri)
                .await?
        };

        log::debug!(
            "Resolved target: transport={}, addr={}",
            selected_transport.transport_type(),
            destination
        );

        let outgoing_info = OutgoingMessageInfo {
            destination,
            transport: selected_transport,
        };

        let outgoing = OutgoingRequest::new(request, outgoing_info);
        
        let transaction = Transaction::new_client(&outgoing, endpoint)?;

        endpoint.send_request(&outgoing).await?;

        Ok(Self {
            transaction,
            request: Arc::new(outgoing),
        })

        // let transaction = Transaction::new_client(&outgoing, selected_transport, destination);

        // transaction.send_request

        // PrependHeader
        // find_hdr_by_name

        // ============================================================
        // RFC 3261 §8.1.2 — Sending the Request
        // ============================================================
        //
        // Determine the next-hop target logically:
        // 1. If Route header exists → use topmost Route.
        // 2. Otherwise → use the Request-URI.
        //
        // After determining the logical target, we apply RFC 3263
        // to resolve its transport and IP.
        // ============================================================

        // let target_uri = outgoing.next_hop_uri()?; // handle Route vs Request-URI

        // ============================================================
        // RFC 3263 §4.1 — Selecting a Transport Protocol
        // ============================================================
        //
        // Decide which transport to use:
        // - If URI has "transport=xxx", use it.
        // - If URI is "sips:", use TLS over TCP.
        // - Otherwise, follow SRV fallback: _sip._udp, _sip._tcp, ...
        // ============================================================

        // let (selected_transport, destination) = if let Some((tr, addr)) = transport {
        //     // Transport and address explicitly provided
        //     (tr, addr)
        // } else {
        //     // Otherwise, resolve according to RFC 3263
        //     let resolved = resolve_target_via_dns(&target_uri).await?;
        //     (resolved.transport, resolved.addr)
        // };

        // ============================================================
        // RFC 3263 §4.2 — Determining Port and IP Address
        // ============================================================
        //
        // If the URI has:
        // - maddr → use that as host
        // - port → use that directly
        // - otherwise, perform SRV → A/AAAA resolution
        // ============================================================

        // log::debug!(
        //     "Resolved target: transport={:?}, addr={}",
        //     selected_transport,
        //     destination
        // );

        // ============================================================
        // RFC 3261 §17.1 — Client Transaction
        // ============================================================
        //
        // Create the client transaction for this INVITE request:
        // - Associate transaction key (branch + Call-ID + method)
        // - Initialize state to "Calling"
        // - Handle retransmission (for UDP) and timeouts
        // ============================================================

        // let transaction = Transaction::new_client(&outgoing, selected_transport, destination);

        // ============================================================
        // Send the request over the resolved transport
        // ============================================================
        // endpoint
        //     .send_request(&outgoing, selected_transport, destination)
        //     .await?;

        // ============================================================
        // Return a fully initialized ClientInviteTx
        // ============================================================

        // Ok(Self {
        //     transaction,
        //     request: Arc::new(outgoing),
        // })
    }

    async fn init_retrans_loop(self) -> Result<()> {
        /*
        pin! {
            let timer_b = time::sleep(64 * T1);
            let timer_a = if !self.is_reliable() {
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
        */
        Ok(())
    }

    pub(crate) async fn receive(&self, response: &IncomingResponse) -> Result<bool> {
        /*
        let code = response.message.code();

        match self.get_state() {
            TransactionState::Calling if code.is_provisional() => {
                self.set_state(TransactionState::Proceeding);
            }
            TransactionState::Calling | TransactionState::Proceeding
                if matches!(code.as_u16(), 300..=699) =>
            {
                self.set_state(TransactionState::Completed);
                let ack = self.create_ack(response);

                self.inner.send_request(&ack).await?;
                self.terminate();
            }
            TransactionState::Calling | TransactionState::Proceeding if code.is_final() => {
                self.on_terminated();
            }
            TransactionState::Completed => {
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
        */
        Ok(true)
    }

    fn create_ack<'a>(&self, response: &IncomingResponse) -> OutgoingRequest {
        /*
        let mut via = None;
        let mut cseq = None;
        let mut call_id = None;
        let mut from = None;

        for header in self.request.as_ref().unwrap().headers.iter() {
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
            .message
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

        let uri = self.request.as_ref().unwrap().req_line.uri.clone();

        ack_hdrs.push(Header::Via(via));
        ack_hdrs.push(Header::From(from));
        ack_hdrs.push(Header::To(to));
        ack_hdrs.push(Header::CallId(cid));
        ack_hdrs.push(Header::CSeq(cseq));

        OutgoingRequest {
            message: Request {
                req_line: RequestLine {
                    method: SipMethod::Ack,
                    uri,
                },
                headers: ack_hdrs,
                body: None,
            },
            send_info: None,
            encoded: Bytes::new(),
        }
        */
        todo!()
    }

    pub(super) fn terminate(&self) {
        /*
        if self.is_reliable() {
            self.on_terminated();
        } else {
            // Start timer D
            self.schedule_termination(TIMER_D);
        }
         */
    }
}

impl DerefMut for ClientInviteTx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for ClientInviteTx {
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
    use crate::{
        message::{SipMethod, StatusCode},
        transaction::TransactionLayer,
    };

    const FROM: &str = "sip:alice@127.0.0.1:5060";
    const TO: &str = "sip:bob@127.0.0.1:5060";

    #[tokio::test]
    async fn test_state_calling() {
        // let request = Request::new(SipMethod::Invite, "sip:localhost".parse().unwrap());
        // let endpoint = crate::endpoint::EndpointBuilder::new()
        //     .add_transaction(TransactionLayer::default())
        //     .build();
        // let tx = ClientInviteTx::send_request(&endpoint, request, None).await.unwrap();

        // tx.send_request().await.unwrap();

        // let endpoint = mock::default_endpoint();
        // let request = endpoint
        //     .create_request(OutgoingRequestConfig {
        //         method: SipMethod::Invite,
        //         uri: TO,
        //         from: FROM,
        //         to: TO,
        //         call_id: "12345",
        //         contact: None,
        //         cseq: Some(1),
        //         body: None,
        //     })
        //     .unwrap();

        // // let request = mock::outgoing_request(SipMethod::Invite);

        // let uac_inv = ClientInviteTx::send_request(request, &endpoint)
        //     .await
        //     .unwrap();

        // assert_eq!(uac_inv.get_state(), TransactionState::Calling);
    }

    /*

    #[tokio::test]
    async fn test_state_calling() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Invite);

        let uac_inv = ClientInviteTx::send(request, &endpoint).await.unwrap();

        assert_eq!(uac_inv.get_state(), TransactionState::Calling);
    }

    #[tokio::test]
    async fn test_state_proceeding() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::Trying);

        let uac_inv = ClientInviteTx::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.get_state(), TransactionState::Proceeding);
    }

    #[tokio::test]
    async fn test_state_completed() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::BusyHere);

        let uac_inv = ClientInviteTx::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.last_status_code(), Some(StatusCode::BusyHere));
        assert_eq!(uac_inv.get_state(), TransactionState::Completed);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_a() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Invite);

        let uac_inv = ClientInviteTx::send(request, &endpoint).await.unwrap();

        assert!(uac_inv.retrans_count() == 0);
        assert_eq!(uac_inv.get_state(), TransactionState::Calling);

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
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Invite);

        let uac_inv = ClientInviteTx::send(request, &endpoint).await.unwrap();

        assert_eq!(uac_inv.get_state(), TransactionState::Calling);

        time::sleep(ClientInviteTx::T1 * 64 + Duration::from_millis(1)).await;

        assert!(uac_inv.get_state() == TransactionState::Terminated);
    }

    #[tokio::test(start_paused = true)]
    async fn test_timer_d() {
        let endpoint = mock::default_endpoint();
        let request = mock::outgoing_request(SipMethod::Invite);
        let response = mock::incoming_response(StatusCode::BusyHere);

        let uac_inv = ClientInviteTx::send(request, &endpoint).await.unwrap();

        uac_inv.receive(&response).await.unwrap();

        assert_eq!(uac_inv.get_state(), TransactionState::Completed);

        time::sleep(TIMER_D + Duration::from_millis(1)).await;

        assert!(uac_inv.get_state() == TransactionState::Terminated);
    }
    */
}
