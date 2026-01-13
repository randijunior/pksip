use tokio::sync::{mpsc, watch};

use crate::{
    Method,
    endpoint::EndpointBuilder,
    transaction::{ServerTransaction, TransactionMessage, fsm},
    transport::{IncomingRequest, Transport, mock::MockTransport},
};

const FINAL_NON_2XX_STATUS_CODE: u16 = 301;
const PROVISIONAL_1XX_STATUS_CODE: u16 = 182;

mod invite;
mod non_invite;

use super::{create_test_endpoint_and_request, create_test_request};

fn create_server_transaction(
    method: Method,
    transport: Option<MockTransport>,
) -> (ServerTransaction, watch::Receiver<fsm::State>) {
    let request = create_test_request(method, transport.map(Transport::new));
    let endpoint = EndpointBuilder::new()
        .add_transaction(Default::default())
        .build();

    let mut server_tsx = ServerTransaction::from_request(request, &endpoint).unwrap();
    let state = server_tsx.state_machine_mut().subscribe_state();

    (server_tsx, state)
}

struct MockClientTransaction {
    sender: mpsc::Sender<super::TransactionMessage>,
    request: IncomingRequest,
}

impl MockClientTransaction {
    pub async fn retransmit_to_transaction(&self) {
        self.sender
            .send(TransactionMessage::Request(self.request.clone()))
            .await
            .unwrap();
        tokio::task::yield_now().await;
    }

    pub async fn retransmit_n_times(&self, n: usize) {
        for _ in 0..n {
            self.retransmit_to_transaction().await;
        }
    }

    pub async fn send_ack_request(&mut self) {
        let mut request = self.request.clone();
        request.message.req_line.method = Method::Ack;
        self.sender
            .send(TransactionMessage::Request(request))
            .await
            .unwrap();
        tokio::task::yield_now().await;
    }
}

fn setup_test_server_retransmission(
    method: Method,
) -> (MockClientTransaction, MockTransport, ServerTransaction) {
    let transport = MockTransport::new_udp();
    let transport_clone = transport.clone();

    let (endpoint, request) = create_test_endpoint_and_request(method, transport_clone.into());
    let server = ServerTransaction::from_request(request.clone(), &endpoint).unwrap();

    let sender = endpoint
        .transactions()
        .get_entry(server.transaction_key())
        .unwrap();

    let sender = MockClientTransaction { sender, request };

    (sender, transport, server)
}

fn setup_test_server_state_reliable(
    method: Method,
) -> (ServerTransaction, watch::Receiver<fsm::State>) {
    create_server_transaction(method, Some(MockTransport::new_tcp()))
}

fn setup_test_server_state_unreliable(
    method: Method,
) -> (ServerTransaction, watch::Receiver<fsm::State>) {
    create_server_transaction(method, Some(MockTransport::new_udp()))
}

fn setup_test_server_receive_ack() -> (
    MockClientTransaction,
    watch::Receiver<fsm::State>,
    ServerTransaction,
) {
    let (sender, _, mut server) = setup_test_server_retransmission(Method::Invite);

    (sender, server.state_machine_mut().subscribe_state(), server)
}
