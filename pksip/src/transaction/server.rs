use std::future;

use crate::SipMethod;
use crate::endpoint::Endpoint;
use crate::error::{Result, TransactionError};
use crate::message::headers::Headers;
use crate::message::{ReasonPhrase, SipMessageBody, StatusCode};
use crate::transaction::fsm::{State, StateMachine};
use crate::transaction::manager::TransactionKey;
use crate::transaction::{T1, T2, T4, TransactionMessage};
use crate::transport::incoming::IncomingRequest;
use crate::transport::outgoing::OutgoingResponse;

use tokio::sync::mpsc::{self};
use tokio::time::{Instant, sleep, timeout_at};
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
        if let SipMethod::Ack = request.request.req_line.method {
            return Err(TransactionError::AckCannotCreateTransaction.into());
        }
        let key = TransactionKey::from_request(&request);
        let (main_tx, main_rx) = mpsc::channel(10);

        endpoint
            .transactions()
            .add_transaction(key.clone(), main_tx);
        let state = if request.request.req_line.method == SipMethod::Invite {
            State::Proceeding
        } else {
            State::Trying
        };

        Ok(Self {
            endpoint: endpoint.clone(),
            key,
            request,
            state: StateMachine::new(state),
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
                           .target_info
                           .transport
                           .send_msg(&response.encoded, &response.target_info.target)
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
            response.response.headers.extend(aditional_headers);
        }

        if let Some(body) = body {
            response.response.body = Some(body);
        }

        self.endpoint.send_outgoing_response(&mut response).await?;

        if self.request.request.req_line.method == SipMethod::Invite {
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
                            if req.request.req_line.method.is_ack() {
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
        self.request.incoming_info.transport.transport.is_reliable()
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

/// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    use crate::assert_eq_state;

    use crate::test_utils::{
        CODE_100_TRYING, CODE_202_ACCEPTED, CODE_301_MOVED_PERMANENTLY, CODE_504_SERVER_TIMEOUT,
    };

    use crate::test_utils::transaction::ServerTestContext;

    // INVITE Server tests

    #[tokio::test]
    async fn invite_transitions_to_proceeding_when_created_from_request() {
        let mut ctx = ServerTestContext::setup(SipMethod::Invite);

        assert_eq_state!(
            ctx.state,
            State::Proceeding,
            "server INVITE must transition to the Proceeding state when constructed for a request"
        );
    }

    #[tokio::test]
    async fn invite_transitions_to_confirmed_when_receiving_ack() {
        let mut ctx = ServerTestContext::setup(SipMethod::Invite);

        ctx.server
            .respond_final_code(CODE_301_MOVED_PERMANENTLY)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Completed,
            "server INVITE must transition to the Completed state when sending 200-699 response"
        );

        ctx.client.send_ack_request().await;

        assert_eq_state!(
            ctx.state,
            State::Confirmed,
            "server INVITE must transition to the Confirmed state when receiving the ACK request"
        );
    }

    #[tokio::test]
    async fn invite_unreliable_transitions_to_terminated_when_sending_2xx_response() {
        let mut ctx = ServerTestContext::setup(SipMethod::Invite);

        ctx.server
            .respond_final_code(CODE_202_ACCEPTED)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Terminated,
            "server INVITE must transition to the Terminated state when sending 2xx response"
        );
    }

    #[tokio::test]
    async fn invite_reliable_transitions_to_terminated_when_sending_2xx_response() {
        let mut ctx = ServerTestContext::setup_reliable(SipMethod::Invite);

        ctx.server
            .respond_final_code(CODE_202_ACCEPTED)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Terminated,
            "server INVITE must transition to the Terminated state when sending 2xx response"
        );
    }

    #[tokio::test]
    async fn invite_should_retransmit_response_when_receiving_request_retransmission() {
        let ctx = ServerTestContext::setup(SipMethod::Invite);
        let expected_responses = 1;
        let expected_retrans = 3;

        ctx.server
            .respond_final_code(CODE_301_MOVED_PERMANENTLY)
            .await
            .expect("Error sending final response");

        ctx.client.retransmit_n_times(expected_retrans).await;

        assert_eq!(
            ctx.transport.sent_count(),
            expected_responses + expected_retrans,
            "sent count should match {expected_responses} responses and {expected_retrans} retransmissions"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn invite_must_cease_retransmission_when_receiving_ack() {
        let mut ctx = ServerTestContext::setup(SipMethod::Invite);
        let expected_responses = 1;
        let expected_retrans = 2;

        ctx.server
            .respond_final_code(CODE_301_MOVED_PERMANENTLY)
            .await
            .expect("Error sending final response");

        ctx.timer.wait_for_retransmissions(expected_retrans).await;

        ctx.client.send_ack_request().await;

        // Should not retransmit at this point.
        ctx.timer.wait_for_retransmissions(3).await;

        assert_eq!(
            ctx.transport.sent_count(),
            expected_responses + expected_retrans,
            "sent count should match {expected_responses} responses and {expected_retrans} retransmissions"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn invite_timer_h_must_be_set_for_reliable_transports() {
        let mut ctx = ServerTestContext::setup_reliable(SipMethod::Invite);

        ctx.server
            .respond_final_code(CODE_301_MOVED_PERMANENTLY)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Completed,
            "server INVITE must transition to the Completed state when sending final 200-699 response"
        );

        ctx.timer.timer_h().await;

        assert_eq_state!(
            ctx.state,
            State::Terminated,
            "server INVITE must transition to the Terminated state when timer H fires"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn invite_timer_h_must_be_set_for_unreliable_transports() {
        let mut ctx = ServerTestContext::setup(SipMethod::Invite);

        ctx.server
            .respond_final_code(CODE_301_MOVED_PERMANENTLY)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Completed,
            "server INVITE must transition to the Completed state when sending 200-699 response"
        );

        ctx.timer.timer_h().await;

        assert_eq_state!(
            ctx.state,
            State::Terminated,
            "server INVITE must transition to the Terminated state when timer H fires"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn invite_transitions_to_terminated_when_timer_i_fires() {
        let mut ctx = ServerTestContext::setup(SipMethod::Invite);

        ctx.server
            .respond_final_code(CODE_301_MOVED_PERMANENTLY)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Completed,
            "server INVITE must must transition to the Completed when sending 300-699 response"
        );

        ctx.client.send_ack_request().await;

        assert_eq_state!(
            ctx.state,
            State::Confirmed,
            "server INVITE must transition to the Confirmed state when receiving ACK request"
        );

        ctx.timer.timer_i().await;

        assert_eq_state!(
            ctx.state,
            State::Terminated,
            "server INVITE must transition to the Terminated state when timer I fires"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn invite_retransmit_response_when_timer_g_fires() {
        let mut ctx = ServerTestContext::setup(SipMethod::Invite);
        let expected_responses = 1;
        let expected_retrans = 5;

        ctx.server
            .respond_final_code(CODE_301_MOVED_PERMANENTLY)
            .await
            .expect("Error sending final response");

        ctx.timer.wait_for_retransmissions(expected_retrans).await;

        assert_eq!(
            ctx.transport.sent_count(),
            expected_responses + expected_retrans,
            "sent count should match {expected_responses} requests and {expected_retrans} retransmissions"
        );
    }

    // Non-INVITE Server tests

    #[tokio::test]
    async fn non_invite_transitions_to_trying_when_created_from_request() {
        let mut ctx = ServerTestContext::setup(SipMethod::Options);

        assert_eq_state!(
            ctx.state,
            State::Trying,
            "server non-INVITE must transition to the Trying state when constructed for a request"
        );
    }

    #[tokio::test]
    async fn non_invite_transition_to_proceeding_when_sending_1xx_response() {
        let mut ctx = ServerTestContext::setup(SipMethod::Options);

        ctx.server
            .respond_provisional_code(CODE_100_TRYING)
            .await
            .expect("Error sending provisional response");

        assert_eq_state!(
            ctx.state,
            State::Proceeding,
            "server non-INVITE must transition to the Proceeding state when sending 1xx response"
        );
    }

    #[tokio::test]
    async fn non_invite_transition_to_completed_when_sending_non_2xx_response() {
        let mut ctx = ServerTestContext::setup(SipMethod::Options);

        ctx.server
            .respond_final_code(CODE_504_SERVER_TIMEOUT)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Completed,
            "server non-INVITE must transition to the Completed state when sending 200-699 response"
        );
    }

    #[tokio::test]
    async fn non_invite_reliable_transition_to_terminated_when_sending_2xx_response() {
        let mut ctx = ServerTestContext::setup_reliable(SipMethod::Options);

        ctx.server
            .respond_final_code(CODE_202_ACCEPTED)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Terminated,
            "server non-INVITE must transition to the Terminated state when sending 2xx response"
        );
    }

    #[tokio::test]
    async fn non_invite_reliable_transition_to_terminated_when_sending_non_2xx_response() {
        let mut ctx = ServerTestContext::setup_reliable(SipMethod::Options);

        ctx.server
            .respond_final_code(CODE_504_SERVER_TIMEOUT)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Terminated,
            "server non-INVITE must transition to the Terminated state when sending 2xx response"
        );
    }

    #[tokio::test]
    async fn non_invite_absorbs_retransmission_in_trying_state() {
        let ctx = ServerTestContext::setup(SipMethod::Options);
        let expected_retrans = 0;

        ctx.client.retransmit_n_times(2).await;

        assert_eq!(
            ctx.transport.sent_count(),
            expected_retrans,
            "sent count should match {expected_retrans} retransmissions"
        );
    }

    #[tokio::test]
    async fn non_invite_retransmit_provisional_response_when_receiving_request_retransmission() {
        let mut ctx = ServerTestContext::setup(SipMethod::Options);
        let expected_responses = 1;
        let expected_retrans = 4;

        ctx.server
            .respond_provisional_code(CODE_100_TRYING)
            .await
            .expect("Error sending provisional response");

        ctx.client.retransmit_n_times(expected_retrans).await;

        assert_eq!(
            ctx.transport.sent_count(),
            expected_responses + expected_retrans,
            "sent count should match {expected_responses} responses and {expected_retrans} retransmissions"
        );
    }

    #[tokio::test]
    async fn non_invite_retransmit_final_response_when_receiving_request_retransmission() {
        let ctx = ServerTestContext::setup(SipMethod::Register);
        let expected_responses = 1;
        let expected_retrans = 2;

        ctx.server
            .respond_final_code(CODE_202_ACCEPTED)
            .await
            .expect("Error sending final response");

        ctx.client.retransmit_n_times(expected_retrans).await;

        assert_eq!(
            ctx.transport.sent_count(),
            expected_responses + expected_retrans,
            "sent count should match {expected_responses} responses and {expected_retrans} retransmissions"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn non_invite_transitions_to_terminated_when_timer_j_fires() {
        let mut ctx = ServerTestContext::setup(SipMethod::Bye);

        ctx.server
            .respond_final_code(CODE_202_ACCEPTED)
            .await
            .expect("Error sending final response");

        assert_eq_state!(
            ctx.state,
            State::Completed,
            "server non-INVITE must must transition to the Completed state when sending 200-699 response"
        );

        ctx.timer.timer_j().await;

        assert_eq_state!(
            ctx.state,
            State::Terminated,
            "server non-INVITE must transition to the Terminated state when timer J fires"
        );
    }
}
