use std::{net::SocketAddr, time::Duration};

use crate::{
    Endpoint, Method, Result,
    error::TransactionError,
    find_map_mut_header,
    message::{
        Request,
        headers::{Header, Via},
    },
    transaction::{
        PeekableReceiver, Role,
        fsm::{State, StateMachine},
        manager::TransactionKey,
    },
    transport::{IncomingResponse, OutgoingRequest, Transport},
};

use super::{T1, T4, TransactionMessage};
use tokio::{
    sync::{
        mpsc::{self},
    },
    time::{Instant, timeout, timeout_at},
};

// ACK para 2xx é responsabilidade do TU.

/// An Client Transaction, either `Invite` or `NonInvite`.
pub struct ClientTransaction {
    key: TransactionKey,
    endpoint: Endpoint,
    state_machine: StateMachine,
    request: OutgoingRequest,
    channel: PeekableReceiver<TransactionMessage>,
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
        let headers = &mut request.message.headers;
        let via = match find_map_mut_header!(headers, Via) {
            Some(via) => via,
            None => {
                let sent_by = request.send_info.transport.local_addr().into();
                let transport = request.send_info.transport.protocol();
                let branch = crate::generate_branch(None);
                let via = Via::new_with_transport(transport, sent_by, Some(branch));

                headers.prepend_header(Header::Via(via));

                match headers.first_mut().unwrap() {
                    Header::Via(v) => v,
                    _ => unreachable!(),
                }
            }
        };
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
            State::Calling
        } else {
            State::Trying
        };
        let (sender, receiver) = mpsc::channel(10);

        endpoint.transactions().add_transaction(key.clone(), sender);

        let uac = ClientTransaction {
            key,
            endpoint: endpoint.clone(),
            state_machine: StateMachine::new(state),
            channel: receiver.into(),
            request,
            timeout: Instant::now() + T1 * 64,
        };

        log::trace!("Transaction Created [{:#?}] ({:p})", Role::UAC, &uac);

        Ok(uac)
    }

    pub fn state(&self) -> State {
        self.state_machine.state()
    }

    pub fn state_machine_mut(&mut self) -> &mut StateMachine {
        &mut self.state_machine
    }

    async fn recv_provisional_msg(&mut self) -> Option<IncomingResponse> {
        match self
            .channel
            .recv_if(|msg| match msg {
                TransactionMessage::Response(incoming)
                    if incoming.message.status_code().is_provisional() =>
                {
                    true
                }
                _ => false,
            })
            .await
        {
            Some(TransactionMessage::Response(provisional_response)) => {
                return Some(provisional_response);
            }
            _ => return None,
        }
    }

    pub async fn receive_provisional_response(&mut self) -> Result<Option<IncomingResponse>> {
        match self.state_machine.state() {
            State::Trying | State::Calling if !self.request.send_info.transport.is_reliable() => {
                let mut retrans_interval = T1;
                loop {
                    let timer = self.timeout.into();
                    let msg = timeout(retrans_interval, self.recv_provisional_msg());

                    match timeout_at(timer, msg).await {
                        Ok(Ok(Some(msg))) => {
                            self.state_machine.set_state(State::Proceeding);
                            return Ok(Some(msg));
                        }
                        Ok(Err(_)) => {
                            // retransmit
                            self.endpoint
                                .send_outgoing_request(&mut self.request)
                                .await?;
                            retrans_interval *= 2;
                            continue;
                        }
                        Err(_elapsed) => {
                            self.state_machine.set_state(State::Terminated);
                            return Err(TransactionError::Timeout.into());
                        }
                        _ => todo!(),
                    }
                }
            }
            State::Initial => {}
            State::Calling => {
                match timeout_at(self.timeout.into(), self.recv_provisional_msg()).await {
                    Ok(Some(msg)) => {
                        self.state_machine.set_state(State::Proceeding);
                        return Ok(Some(msg));
                    }
                    Ok(None) => return Ok(None),
                    Err(_elapsed) => {
                        self.state_machine.set_state(State::Terminated);
                        return Err(TransactionError::Timeout.into());
                    }
                }
            }
            State::Trying => todo!(),
            State::Proceeding => {
                // TODO: Add Timeout
                return Ok(self.recv_provisional_msg().await);
            }
            State::Completed => todo!(),
            State::Confirmed => todo!(),
            State::Terminated => todo!(),
        }
        todo!()
    }

    pub async fn receive_final_response(mut self) -> Result<IncomingResponse> {
        // Change to only receive final.
        let response = self.channel.recv().await.unwrap();

        let TransactionMessage::Response(response) = response else {
            unimplemented!()
        };

        if self.request.message.req_line.method == Method::Invite
            && let 200..299 = response.message.status_line.code.as_u16()
            && matches!(
                self.state_machine.state(),
                State::Calling | State::Proceeding
            )
        {
            self.state_machine.set_state(State::Terminated);
            return Ok(response);
        }
        self.state_machine.set_state(State::Completed);

        if self.is_reliable() {
            self.state_machine.set_state(State::Terminated);
            return Ok(response);
        }

        if self.request.message.req_line.method == Method::Invite {
            // send ACK
            let mut ack_request = self.endpoint.create_ack_request(&self.request, &response);
            self.endpoint
                .send_outgoing_request(&mut ack_request)
                .await?;

            // timer d fires
            let timer_d = Instant::now() + 64 * T1;
            tokio::spawn(async move {
                while let Ok(Some(_)) = timeout_at(timer_d, self.channel.recv()).await {
                    if let Err(err) = self.endpoint.send_outgoing_request(&mut ack_request).await {
                        log::error!("Failed to retransmit: {}", err);
                    }
                }
                self.state_machine.set_state(State::Terminated);
            });
        } else {
            // timer k fires
            let timer_k = Instant::now() + T4;
            tokio::spawn(async move {
                while let Ok(Some(_)) = timeout_at(timer_k, self.channel.recv()).await {
                    // buffer any additional response retransmissions that may be received
                }
                self.state_machine.set_state(State::Terminated);
            });
        }

        Ok(response)
    }

    pub fn transaction_key(&self) -> &TransactionKey {
        &self.key
    }

    fn is_reliable(&self) -> bool {
        self.request.send_info.transport.is_reliable()
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
        let state = self.state_machine.state();

        if matches!(state, State::Trying | State::Calling) {
            self.state_machine.set_state(State::Proceeding);
        }

        if matches!(state, State::Completed) {
            // self.retransmit(None).await?;
        }

        Ok(None)
    }
}

impl Drop for ClientTransaction {
    fn drop(&mut self) {
        self.endpoint.transactions().remove(&self.key);
        log::trace!("Transaction Destroyed [{:#?}] ({:p})", Role::UAC, &self);
    }
}
