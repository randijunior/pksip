use std::ops::Deref;
use std::ops::DerefMut;

use crate::core::SipEndpoint;
use crate::error::Result;
use crate::message::SipMethod;
use crate::transaction::State;
use crate::transaction::Transaction;
use crate::transaction::T1;
use crate::transport::IncomingRequest;
use crate::transport::OutgoingResponse;

/// Represents a Server Non INVITE transaction.
#[derive(Clone)]
pub struct ServerTransaction {
    transaction: Transaction,
}

impl ServerTransaction {
    pub(crate) fn new(endpoint: &SipEndpoint, request: &IncomingRequest) -> Self {
        let transactions = endpoint.transactions();
        let method = request.method();

        assert!(
            !matches!(method, SipMethod::Ack | SipMethod::Invite),
            "Invalid request method: {}. ACK and INVITE are not allowed here.",
            method
        );

        let transaction = Transaction::new_uas(request, endpoint);
        let uas = Self { transaction };

        transactions.add_server_tsx_to_map(uas.clone());

        uas
    }

    /// Send a response and update the state.
    pub async fn respond(&self, msg: &mut OutgoingResponse) -> Result<()> {
        self.tsx_send_response(msg).await?;

        match self.get_state() {
            State::Trying if msg.is_provisional() => {
                self.change_state_to(State::Proceeding);
            }
            State::Trying | State::Proceeding => {
                self.change_state_to(State::Completed);
                self.terminate();
            }
            _ => (),
        }

        Ok(())
    }

    pub(super) fn terminate(&self) {
        if self.reliable() {
            self.on_terminated();
        } else {
            self.schedule_termination(T1 * 64);
        }
    }
}

impl DerefMut for ServerTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for ServerTransaction {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;
    use tokio::time::{self};

    use super::*;
    use crate::message::SipMethod;
    use crate::message::StatusCode;
    use crate::transaction::mock;

    #[tokio::test]
    async fn test_receives_100_trying() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint().await;
        let tsx = ServerTransaction::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().as_u16() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint().await;
        let tsx = ServerTransaction::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Ok);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().as_u16() == 200);
        assert!(tsx.get_state() == State::Completed);
    }

    #[tokio::test]
    async fn test_proceeding() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint().await;
        let tsx = ServerTransaction::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().as_u16() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test(start_paused = true)]
    async fn test_terminated_timer_j() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint().await;
        let tsx = ServerTransaction::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Ok);

        tsx.respond(response).await.unwrap();

        time::sleep(T1 * 64 + Duration::from_millis(1)).await;

        assert!(tsx.last_status_code().unwrap().as_u16() == 200);
        assert!(tsx.get_state() == State::Terminated);
    }
}
