use std::ops::{Deref, DerefMut};

use crate::{
    endpoint::Endpoint,
    error::Result,
    message::SipMethod,
    transaction::{T1, sip_transaction::{Transaction, TransactionState}},
    transport::{IncomingRequest, OutgoingResponse},
};

/// Represents a Server Non INVITE transaction.
#[derive(Clone)]
pub struct ServerNonInviteTx {
    transaction: Transaction,
}

impl ServerNonInviteTx {
    pub(crate) fn new(endpoint: &Endpoint, request: &IncomingRequest) -> Self {
        let method = request.message.method();

        assert!(
            !matches!(method, SipMethod::Ack | SipMethod::Invite),
            "Invalid request method: {}. ACK and INVITE are not allowed here.",
            method
        );

        let transaction = Transaction::create_server(request, endpoint).unwrap();
        let uas = Self { transaction };

        endpoint.transactions().add_server_tsx_to_map(uas.clone());

        uas
    }

    /// Send a response and update the state.
    pub async fn respond(&self, msg: OutgoingResponse) -> Result<()> {
        /*
        let is_provisional = msg.message.status_line.code.is_provisional();
        self.send_response(msg).await?;

        match self.get_state() {
            TransactionState::Trying if is_provisional => {
                self.set_state(TransactionState::Proceeding);
            }
            TransactionState::Trying | TransactionState::Proceeding => {
                self.set_state(TransactionState::Completed);
                self.terminate();
            }
            _ => (),
        }
         */

        Ok(())
    }

    pub(super) fn terminate(&self) {
        /*
        if self.is_reliable() {
            self.on_terminated();
        } else {
            self.schedule_termination(T1 * 64);
        }
         */
    }
}

impl DerefMut for ServerNonInviteTx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transaction
    }
}

impl Deref for ServerNonInviteTx {
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
    use crate::message::{SipMethod, StatusCode};
    /*

    #[tokio::test]
    async fn test_receives_100_trying() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint();
        let tsx = ServerNonInviteTx::new(&endpoint, &mut request);
        let response = mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.get_state() == TransactionState::Proceeding);
    }

    #[tokio::test]
    async fn test_receives_200_ok() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint();
        let tsx = ServerNonInviteTx::new(&endpoint, &mut request);
        let response = mock::response(StatusCode::Ok);

        tsx.respond(response).await.unwrap();

        assert!(tsx.get_state() == TransactionState::Completed);
    }

    #[tokio::test]
    async fn test_proceeding() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint();
        let tsx = ServerNonInviteTx::new(&endpoint, &mut request);
        let response = mock::response(StatusCode::Trying);

        tsx.respond(response).await.unwrap();

        assert!(tsx.get_state() == TransactionState::Proceeding);
    }

    #[tokio::test(start_paused = true)]
    async fn test_terminated_timer_j() {
        let mut request = mock::request(SipMethod::Options);
        let endpoint = mock::default_endpoint();
        let tsx = ServerNonInviteTx::new(&endpoint, &mut request);
        let response = mock::response(StatusCode::Ok);

        tsx.respond(response).await.unwrap();

        time::sleep(T1 * 64 + Duration::from_millis(1)).await;

        assert!(tsx.get_state() == TransactionState::Terminated);
    }
     */
}
