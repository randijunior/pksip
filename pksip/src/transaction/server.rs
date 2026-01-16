use std::future;

use crate::{
    SipMethod,
    endpoint::Endpoint,
    error::{Result, TransactionError},
    message::{ReasonPhrase, SipMessageBody, StatusCode, headers::Headers},
    transaction::{
        T1, T2, T4, TransactionMessage,
        fsm::{State, StateMachine},
        manager::TransactionKey,
    },
    transport::{IncomingRequest, OutgoingResponse},
};

use tokio::{
    sync::mpsc::{self},
    time::{Instant, sleep, timeout_at},
};
use tokio_util::either::Either;

pub struct ServerTransaction {
    key: TransactionKey,
    endpoint: Endpoint,
    state: StateMachine,
    request: IncomingRequest,
    receiver: Option<mpsc::Receiver<TransactionMessage>>,
    proceeding_state_task: Option<ProceedingStateTask>,
}

impl ServerTransaction {
    pub fn from_request(request: IncomingRequest, endpoint: &Endpoint) -> Result<Self> {
        if let SipMethod::Ack = request.req_line.method {
            return Err(TransactionError::AckCannotCreateTransaction.into());
        }
        let (main_tx, main_rx) = mpsc::channel(10);
        let key = TransactionKey::from_request(&request);

        endpoint
            .transactions()
            .add_transaction(key.clone(), main_tx);

        Ok(Self {
            endpoint: endpoint.clone(),
            key,
            request,
            state: StateMachine::new(State::Initial),
            proceeding_state_task: None,
            receiver: Some(main_rx),
        })
    }

    pub fn state_machine(&self) -> &StateMachine {
        &self.state
    }

    pub fn state_machine_mut(&mut self) -> &mut StateMachine {
        &mut self.state
    }

    pub async fn respond_provisional_code(&mut self, code: impl TryInto<StatusCode>) -> Result<()> {
        self.send_provisional_response(code, None, None, None).await
    }

    pub async fn send_provisional_response(
        &mut self,
        code: impl TryInto<StatusCode>,
        phrase: Option<ReasonPhrase>,
        headers: Option<Headers>,
        body: Option<SipMessageBody>,
    ) -> Result<()> {
        let code = StatusCode::try_new_provisional(code)?;

        let mut response = self.endpoint.create_response(&self.request, code, phrase);

        self.endpoint.send_outgoing_response(&mut response).await?;

        if self.state.state() != State::Proceeding {
            self.state.set_state(State::Proceeding);
        }

        if let Some(ref mut task) = self.proceeding_state_task {
            task.tu_provisional_tx.send(response).ok();
            return Ok(());
        }

        let mut receiver = self
            .receiver
            .take()
            .expect("The transaction rx should exists");

        let (tu_provisional_tx, mut tu_provisional_receiver) = mpsc::unbounded_channel();

        let mut state_rx = self.state.subscribe_state();

        let proceeding_state_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased; // Ensures polling order from top to bottom
                    _= state_rx.changed() => {
                        // Leave Proceding State
                        log::debug!("Task is cancelling...");
                        return receiver;
                    }
                    Some(new_tu_provisional) = tu_provisional_receiver.recv() => {
                        response = new_tu_provisional;
                    }
                    Some(_) = receiver.recv() => {
                           if let Err(err) = response
                           .send_info
                           .transport
                           .send_msg(&response.encoded, &response.send_info.target)
                           .await {
                            log::error!("Failed to retransmit: {}", err);
                           }
                    }
                }
            }
        });

        self.proceeding_state_task = Some(ProceedingStateTask {
            tu_provisional_tx,
            proceeding_state_task,
        });

        Ok(())
    }

    pub async fn respond_final_code(mut self, code: StatusCode) -> Result<()> {
        self.send_final_response(code, None, None, None).await
    }

    pub async fn send_final_response(
        mut self,
        code: StatusCode,
        phrase: Option<ReasonPhrase>,
        headers: Option<Headers>,
        body: Option<SipMessageBody>,
    ) -> Result<()> {
        if !code.is_final() {
            return Err(TransactionError::InvalidFinalStatusCode.into());
        }

        let mut response = self.endpoint.create_response(&self.request, code, phrase);

        if let Some(aditional_headers) = headers {
            response.message.headers.extend(aditional_headers);
        }

        if let Some(body) = body {
            response.message.body = Some(body);
        }

        self.endpoint.send_outgoing_response(&mut response).await?;

        if self.request.message.req_line.method == SipMethod::Invite {
            if let 200..299 = code.as_u16() {
                self.state.set_state(State::Terminated);
                return Ok(());
            }
            // 300-699 from TU send response --> Completed
            self.state.set_state(State::Completed);

            let mut receiver = if let Some(task) = self.proceeding_state_task.take() {
                task.proceeding_state_task.await.unwrap()
            } else {
                self.receiver.take().unwrap()
            };

            // For unreliable transports.
            let timer_g = if !self.is_reliable() {
                Either::Left(sleep(T1))
            } else {
                Either::Right(future::pending::<()>())
            };
            // For all transports.
            let timer_h = sleep(64 * T1);
            let mut retrans_count = 0;
            tokio::spawn(async move {
                tokio::pin!(timer_g);
                tokio::pin!(timer_h);
                loop {
                    tokio::select! {
                        _ = timer_g.as_mut() => {
                           let _res =  self.endpoint
                            .send_outgoing_response(&mut response)
                            .await;
                        retrans_count += 1;

                        let new_timer = T1 * (1 << retrans_count);
                        let sleep = sleep(std::cmp::min(new_timer, T2));

                        timer_g.set(Either::Left(sleep));

                        continue;

                        }
                        _ = timer_h.as_mut() => {
                            // Timeout
                            self.state.set_state(State::Terminated);
                            return;
                        }
                         Some(TransactionMessage::Request(req)) = receiver.recv() => {
                            if req.message.req_line.method.is_ack() {
                                self.state.set_state(State::Confirmed);
                                sleep(T4).await;
                                self.state.set_state(State::Terminated);
                                return;
                            }
                            let _res =  self.endpoint
                            .send_outgoing_response(&mut response)
                            .await;
                        }
                    }
                }
            });
        } else {
            // 200-699 from TU send response --> Completed
            self.state.set_state(State::Completed);

            if self.is_reliable() {
                self.state.set_state(State::Terminated);
                return Ok(());
            }

            let mut receiver = if let Some(task) = self.proceeding_state_task.take() {
                task.proceeding_state_task.await.unwrap()
            } else {
                self.receiver.take().unwrap()
            };

            let timer_j = Instant::now() + 64 * T1;

            tokio::spawn(async move {
                while let Ok(Some(_)) = timeout_at(timer_j, receiver.recv()).await {
                    let _result = self.endpoint.send_outgoing_response(&mut response).await;
                }
                self.state.set_state(State::Terminated);
            });
        }

        Ok(())
    }

    pub fn transaction_key(&self) -> &TransactionKey {
        &self.key
    }

    fn is_reliable(&self) -> bool {
        self.request.info.transport.transport.is_reliable()
    }
}

impl Drop for ServerTransaction {
    fn drop(&mut self) {
        self.endpoint.transactions().remove(&self.key);
    }
}

struct ProceedingStateTask {
    proceeding_state_task: tokio::task::JoinHandle<mpsc::Receiver<TransactionMessage>>,
    tu_provisional_tx: mpsc::UnboundedSender<OutgoingResponse>,
}
