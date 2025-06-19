use async_trait::async_trait;

use crate::{
    endpoint::Endpoint,
    error::Result,
    message::SipMethod,
    transaction::{SipTransaction, State, Transaction},
    transport::{IncomingRequest, OutgoingResponse},
};
use std::ops::{Deref, DerefMut};

/// Represents a Server Non INVITE transaction.
#[derive(Clone)]
pub struct TsxUas {
    transaction: Transaction,
}

impl TsxUas {
    pub(crate) fn new(endpoint: &Endpoint, request: &mut IncomingRequest) -> TsxUas {
        let method = request.method();

        assert!(
            !matches!(method, SipMethod::Ack | SipMethod::Invite),
            "Invalid request method: {}. ACK and INVITE are not allowed here.",
            method
        );

        let tsx_layer = endpoint.get_tsx_layer();
        let transaction = Transaction::create_uas(request, endpoint);

        let uas = TsxUas { transaction };

        tsx_layer.add_server_tsx_to_map(uas.clone());
        request.set_tsx(uas.clone());

        uas
    }

    pub async fn respond(&self, msg: &mut OutgoingResponse<'_>) -> Result<()> {
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
}

#[async_trait]
impl SipTransaction for TsxUas {
    fn terminate(&self) {
        if self.reliable() {
            self.on_terminated();
        } else {
            self.schedule_termination(Self::T1 * 64);
        }
    }
}

impl DerefMut for TsxUas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for TsxUas {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{self, Duration};

    use super::*;
    use crate::{
        message::{SipMethod, StatusCode},
        transaction::mock,
    };

    #[tokio::test]
    async fn test_receives_100_trying() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint().await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint().await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Ok);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 200);
        assert!(tsx.get_state() == State::Completed);
    }

    #[tokio::test]
    async fn test_proceeding() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint().await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test(start_paused = true)]
    async fn test_terminated_timer_j() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint().await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Ok);

        tsx.respond(response).await.unwrap();

        time::sleep(TsxUas::T1 * 64 + Duration::from_millis(1)).await;

        assert!(tsx.last_status_code().unwrap().into_i32() == 200);
        assert!(tsx.get_state() == State::Terminated);
    }
}
