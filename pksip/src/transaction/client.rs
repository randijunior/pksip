use std::{net::SocketAddr, time::Duration};

use crate::{
    Endpoint, Method, Result,
    error::TransactionError,
    find_map_mut_header,
    message::Request,
    transaction::{
        Role,manager::TransactionKey,
    },
    transport::{IncomingResponse, OutgoingRequest, Transport},
};

use super::{
    T1, T4, TransactionMessage,
    TransactionState::{self, *},
};
use tokio::{
    sync::mpsc::{self},
    time::{Instant, timeout, timeout_at},
};

const TIMER_D: Duration = Duration::from_secs(32);
const TIMER_K: Duration = T4;

/// An Client Transaction, either `Invite` or `NonInvite`.
pub struct ClientTransaction {
    key: TransactionKey,
    endpoint: Endpoint,
    state: TransactionState,
    request: OutgoingRequest,
    channel: mpsc::UnboundedReceiver<TransactionMessage>,
    timeout: Instant,
}

impl ClientTransaction {
    pub async fn send_request(
        endpoint: &Endpoint,
        request: Request,
        target: Option<(Transport, SocketAddr)>,
    ) -> Result<Self> {
        let method = request.req_line.method;
        if let Method::Ack = method {
            return Err(TransactionError::AckCannotCreateTransaction.into());
        }
        let mut request = endpoint.create_outgoing_request(request, target).await?;
        let message = &mut request.message;
        let header_via = find_map_mut_header!(message.headers, Via);
        let via = header_via.expect("Via header must be present in outgoing request");
        let branch = match via.branch.clone() {
            Some(branch) => branch,
            None => {
                let branch = crate::generate_branch(None);
                via.branch = Some(branch.clone());
                branch
            }
        };
        let key = TransactionKey::new_key_3261(Role::UAC, method, branch);

        endpoint.send_outgoing_request(&mut request).await?;

        let state = if method == Method::Invite {
            TransactionState::Calling
        } else {
            TransactionState::Trying
        };
        let (sender, receiver) = mpsc::unbounded_channel();

        endpoint.transactions().add_transaction(key.clone(), sender);

        let uac = ClientTransaction {
            key,
            endpoint: endpoint.clone(),
            state,
            channel: receiver,
            request,
            timeout: Instant::now() + T1 * 64,
        };

        log::trace!("Transaction Created [{:#?}] ({:p})", Role::UAC, &uac);

        Ok(uac)
    }

    pub async fn receive_provisional_response(&mut self) -> Result<Option<IncomingResponse>> {
        match self.state {
            Initial | Calling if !self.request.send_info.transport.is_reliable() => {
                let mut timer = T1;
                loop {
                    let msg = timeout(timer, self.channel.recv());

                    match timeout_at(self.timeout.into(), msg).await {
                        Ok(Ok(Some(TransactionMessage::Response(msg)))) => {
                            return self.process_response(msg).await;
                        }
                        Ok(Err(_)) => {
                            // retransmit
                            self.endpoint
                                .send_outgoing_request(&mut self.request)
                                .await?;
                            timer *= 2;
                            continue;
                        }
                        Err(_elapsed) => todo!("Timeout"),
                        _ => todo!(),
                    }
                }
            }
            Initial => {}
            Calling => todo!(),
            Trying => todo!(),
            Proceeding => todo!(),
            Completed => todo!(),
            Confirmed => todo!(),
            Terminated => todo!(),
        }
        todo!()
    }

    pub async fn receive_final_response(mut self) -> Result<IncomingResponse> {
        todo!()
    }

    /*
    fn spawn_timer_task(&self) {
        let __self = self.clone();
        tokio::spawn(async move {
            let unreliable = __self.is_unreliable();
            // Invite: Timer A, Non Invite: Timer E
            let retrans_timer = if unreliable {
                Either::Left(time::sleep(T1))
            } else {
                Either::Right(future::pending::<()>())
            };
            // Invite: Timer B, Non Invite: Timer F
            let timeout_timer = time::sleep(64 * T1);
            let completed_state = if let Some(completed) = __self.completed_state_notify() {
                Either::Left(async move { completed.notified().await })
            } else {
                Either::Right(future::pending::<()>())
            };

            tokio::pin!(completed_state);
            tokio::pin!(retrans_timer);
            tokio::pin!(timeout_timer);
            loop {
                tokio::select! {
                    _ = &mut retrans_timer, if matches!(__self.state(), Calling | Trying) => {
                        __self.retransmit(Some(&mut retrans_timer)).await.expect("must retransmit");
                    }
                    _ = &mut timeout_timer, if matches!(__self.state(), Calling | Trying) => {
                        __self.terminate();
                        break;
                    }
                    _ = &mut completed_state => {
                        if unreliable {
                            let duration = if __self.is_invite() { TIMER_D } else { TIMER_K };

                            time::sleep(duration).await;
                        }
                        __self.terminate();
                        break;
                    }
                }
            }
        });
    }
    */

    pub(crate) async fn process_response(
        &mut self,
        response: IncomingResponse,
    ) -> Result<Option<IncomingResponse>> {
        let status_code = response.message.status_code();

        if matches!(self.state, Trying | Calling) {
            self.state = Proceeding;
        }

        if matches!(self.state, Completed) {
            // self.retransmit(None).await?;
        }

        Ok(None)
    }
}

impl Drop for ClientTransaction {
    fn drop(&mut self) {
        self.endpoint.transactions().remove(&self.key);
    }
}

#[cfg(tests)]
mod tests {
    // #[tokio::test]
    // async fn test_client_state_calling() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Invite, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     assert_eq!(client.state(), TransactionState::Calling);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_client_state_proceeding() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Invite, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     let response = Response::from_code(100)?;
    //     client.receive_response(&response).await?;

    //     assert_eq!(client.state(), TransactionState::Proceeding);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_client_state_completed() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Invite, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     let response = Response::from_code(486)?;
    //     client.receive_response(&response).await?;

    //     assert_eq!(client.state(), TransactionState::Completed);

    //     Ok(())
    // }

    // #[tokio::test(start_paused = true)]
    // async fn test_client_timer_a() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Invite, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;
    //     assert_eq!(client.retrans_count(), 0);

    //     time::sleep(Duration::from_millis(500 + 1)).await;
    //     assert_eq!(client.retrans_count(), 1);

    //     time::sleep(Duration::from_secs(1) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 2);

    //     time::sleep(Duration::from_secs(2) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 3);

    //     time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 4);

    //     time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 5);

    //     time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 6);

    //     Ok(())
    // }

    // #[tokio::test(start_paused = true)]
    // async fn test_client_timer_b() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Invite, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;
    //     assert_eq!(client.state(), TransactionState::Calling);

    //     time::sleep(transaction::T1 * 64 + Duration::from_millis(1)).await;
    //     assert_eq!(client.state(), TransactionState::Terminated);

    //     Ok(())
    // }

    // #[tokio::test(start_paused = true)]
    // async fn test_client_timer_d() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Invite, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     let response = Response::from_code(486)?;

    //     client.receive_response(&response).await?;
    //     assert_eq!(client.state(), TransactionState::Completed);

    //     time::sleep(Duration::from_secs(32) + Duration::from_millis(1)).await;
    //     assert!(client.state() == TransactionState::Terminated);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_client_state_trying() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Options, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     assert_eq!(client.state(), TransactionState::Trying);

    //     Ok(())
    // }

    // #[tokio::test(start_paused = true)]
    // #[test_log::test]
    // async fn test_timer_f() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Options, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     time::sleep(transaction::T1 * 64 + Duration::from_millis(1)).await;
    //     assert_eq!(client.state(), TransactionState::Terminated);

    //     Ok(())
    // }

    // #[tokio::test(start_paused = true)]
    // async fn test_fire_timer_k() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Options, uri);
    //     let response = Response::from_code(200)?;

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;
    //     client.receive_response(&response).await?;

    //     time::sleep(transaction::T4 + Duration::from_millis(1)).await;
    //     assert_eq!(client.state(), TransactionState::Terminated);

    //     Ok(())
    // }

    // #[tokio::test(start_paused = true)]
    // async fn test_timer_e_retransmission() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Options, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     assert_eq!(client.retrans_count(), 0);
    //     assert_eq!(client.state(), TransactionState::Trying);
    //     // 500 ms
    //     time::sleep(Duration::from_millis(500 + 1)).await;
    //     assert_eq!(client.retrans_count(), 1);
    //     // 1 s
    //     time::sleep(Duration::from_secs(1) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 2);
    //     // 2 s
    //     time::sleep(Duration::from_secs(2) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 3);
    //     // 4s
    //     time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 4);
    //     // 4s
    //     time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 5);
    //     // 4s
    //     time::sleep(Duration::from_secs(4) + Duration::from_millis(1)).await;
    //     assert_eq!(client.retrans_count(), 6);

    //     assert_eq!(client.state(), TransactionState::Trying);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_client_receives_100_trying() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Options, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     assert_eq!(client.state(), TransactionState::Trying);

    //     let response = Response::from_code(100)?;
    //     client.receive_response(&response).await?;

    //     assert_eq!(client.state(), TransactionState::Proceeding);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_client_receives_200_ok() -> Result<()> {
    //     let (endpoint, uri) = get_test_endpoint(None).await?;
    //     let request = Request::new(Method::Options, uri);

    //     let client = ClientTransaction::send_request(&endpoint, request, None).await?;

    //     assert_eq!(client.state(), TransactionState::Trying);

    //     let response = Response::from_code(200)?;

    //     client.receive_response(&response).await?;
    //     assert!(client.state() == TransactionState::Completed);

    //     Ok(())
    // }
}
