use async_trait::async_trait;

use crate::{
    endpoint::Endpoint,
    error::Result,
    message::Method,
    transaction::{ServerTsx, SipTransaction, State, Transaction},
    transport::{IncomingRequest, OutgoingResponse},
};
use std::ops::{Deref, DerefMut};

/// Represents a Server Non INVITE transaction.
#[derive(Clone)]
pub struct TsxUas {
    tsx: Transaction,
}

impl TsxUas {
    pub(crate) fn new(endpoint: &Endpoint, request: &mut IncomingRequest) -> Self {
        assert!(
            !matches!(request.method(), Method::Ack | Method::Cancel | Method::Invite),
            "Request method cannot be Ack, Cancel or Invite",
        );
        let tsx_layer = endpoint.get_tsx_layer();
        let tsx = Transaction::create_uas(request, endpoint);
        let server_tsx = TsxUas { tsx };

        {
            let server_tsx = server_tsx.clone();
            let server_tsx = ServerTsx::NonInvite(server_tsx);
            request.tsx = Some(server_tsx);
        }

        tsx_layer.new_server_tsx(server_tsx.clone());

        server_tsx
    }

    #[allow(unused_variables)]
    pub(crate) async fn recv_msg<'a>(&self, msg: &IncomingRequest<'a>) -> Result<()> {
        if matches!(self.get_state(), State::Proceeding | State::Completed) {
            self.retransmit().await?;
        }

        Ok(())
    }

    pub async fn respond<'a>(&self, msg: &mut OutgoingResponse<'a>) -> Result<()> {
        self.tsx_send_msg(msg).await?;

        match self.get_state() {
            State::Trying if msg.is_provisional() => {
                self.set_state(State::Proceeding);
            }
            State::Trying | State::Proceeding => {
                self.set_state(State::Completed);
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
        &mut self.tsx
    }
}

impl Deref for TsxUas {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.tsx
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{self, Duration};

    use super::*;
    use crate::{
        endpoint::Builder,
        message::{Method, StatusCode},
        transaction::{mock, TransactionLayer},
    };

    #[tokio::test]
    async fn test_receives_100_trying() {
        let mut request = mock::request(Method::Options);
        let endpoint = Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build()
            .await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let mut request = mock::request(Method::Options);
        let endpoint = Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build()
            .await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Ok);

        tsx.respond(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_i32() == 200);
        assert!(tsx.get_state() == State::Completed);
    }

    #[tokio::test]
    async fn test_retransmit_proceeding() {
        let mut request = mock::request(Method::Options);
        let endpoint = Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build()
            .await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Trying);
        let request = &mock::request(Method::Options);

        tsx.respond(response).await.unwrap();
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.retrans_count() == 1);
        assert!(tsx.last_status_code().unwrap().into_i32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_retransmit_completed() {
        let mut request = mock::request(Method::Options);
        let endpoint = Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build()
            .await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Ok);
        let request = &mock::request(Method::Options);

        tsx.respond(response).await.unwrap();
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.retrans_count() == 1);
        assert!(tsx.last_status_code().unwrap().into_i32() == 200);
        assert!(tsx.get_state() == State::Completed);
    }

    #[tokio::test(start_paused = true)]
    async fn test_terminated_timer_j() {
        let mut request = mock::request(Method::Options);
        let endpoint = Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build()
            .await;
        let tsx = TsxUas::new(&endpoint, &mut request);
        let response = &mut mock::response(StatusCode::Ok);

        tsx.respond(response).await.unwrap();

        time::sleep(TsxUas::T1 * 64 + Duration::from_millis(1)).await;

        assert!(tsx.last_status_code().unwrap().into_i32() == 200);
        assert!(tsx.get_state() == State::Terminated);
    }
}
