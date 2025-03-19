use async_trait::async_trait;

use crate::{
    endpoint::Endpoint,
    message::SipMethod,
    transaction::{SipTransaction, State, Transaction, T1},
    transport::{IncomingMessage, IncomingRequest, OutgoingMessage},
};
use std::{io, ops::Deref};

pub struct UasTsx(Transaction);

impl UasTsx {
    /// Create a new `UasTsx` transaction.
    ///
    /// The state machine is initialized in the [`State::Trying`].
    /// 
    /// # Panics
    ///
    /// This function will panic if the request method is either `Ack` or `Cancel`.
    pub fn new(
        endpoint: &Endpoint,
        request: &IncomingRequest,
    ) -> Self {
        assert!(!matches!(
            request.method(),
            SipMethod::Ack | SipMethod::Cancel
        ));

        Self((request, endpoint).into())
    }
}

#[async_trait]
impl SipTransaction for UasTsx {
    async fn recv_msg(
        &mut self,
        msg: IncomingMessage,
    ) -> io::Result<()> {
        let IncomingMessage::Request(_) = msg else {
            return Ok(());
        };
        match self.get_state() {
            State::Proceeding | State::Completed => {
                self.retransmit().await?
            }
            _ => (),
        }
        Ok(())
    }

    async fn send_msg(
        &mut self,
        msg: OutgoingMessage,
    ) -> io::Result<()> {
        let OutgoingMessage::Response(response) = msg else {
            return Ok(());
        };
        let provisional = response.is_provisional();
        self.send(response).await?;

        match self.get_state() {
            State::Trying if provisional => {
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

    fn terminate(&mut self) {
        if self.reliable() {
            self.on_terminated();
            return;
        }
        let tsx = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(T1 * 64).await;
            tsx.on_terminated();
        });
    }
}

impl Deref for UasTsx {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{self, Duration};

    use super::*;
    use crate::{
        endpoint::EndpointBuilder,
        message::{SipMethod, StatusCode},
        transaction::mock,
    };

    #[tokio::test]
    async fn test_receives_100_trying() {
        let request = mock::request(SipMethod::Options);
        let endpoint = EndpointBuilder::new().build();
        let req = request.request().unwrap();
        let mut tsx = UasTsx::new(&endpoint, &req);
        let response = mock::response(StatusCode::Trying);

        tsx.send_msg(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_u32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let request = mock::request(SipMethod::Options);
        let endpoint = EndpointBuilder::new().build();
        let req = request.request().unwrap();
        let mut tsx = UasTsx::new(&endpoint, &req);
        let response = mock::response(StatusCode::Ok);

        tsx.send_msg(response).await.unwrap();

        assert!(tsx.last_status_code().unwrap().into_u32() == 200);
        assert!(tsx.get_state() == State::Completed);
    }

    #[tokio::test]
    async fn test_retransmit_proceeding() {
        let request = mock::request(SipMethod::Options);
        let endpoint = EndpointBuilder::new().build();
        let req = request.request().unwrap();
        let mut tsx = UasTsx::new(&endpoint, &req);
        let response = mock::response(StatusCode::Trying);

        tsx.send_msg(response).await.unwrap();
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.retransmission_count() == 1);
        assert!(tsx.last_status_code().unwrap().into_u32() == 100);
        assert!(tsx.get_state() == State::Proceeding);
    }

    #[tokio::test]
    async fn test_retransmit_completed() {
        let request = mock::request(SipMethod::Options);
        let endpoint = EndpointBuilder::new().build();
        let req = request.request().unwrap();
        let mut tsx = UasTsx::new(&endpoint, &req);
        let response = mock::response(StatusCode::Ok);

        tsx.send_msg(response).await.unwrap();
        tsx.recv_msg(request).await.unwrap();

        assert!(tsx.retransmission_count() == 1);
        assert!(tsx.last_status_code().unwrap().into_u32() == 200);
        assert!(tsx.get_state() == State::Completed);
    }

    #[tokio::test(start_paused = true)]
    async fn test_terminated_timer_j() {
        let request = mock::request(SipMethod::Options);
        let endpoint = EndpointBuilder::new().build();
        let req = request.request().unwrap();
        let mut tsx = UasTsx::new(&endpoint, &req);
        let response = mock::response(StatusCode::Ok);

        tsx.send_msg(response).await.unwrap();

        time::sleep(T1 * 64 + Duration::from_millis(1)).await;

        assert!(tsx.last_status_code().unwrap().into_u32() == 200);
        assert!(tsx.get_state() == State::Terminated);
    }
}
