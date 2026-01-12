use std::net::SocketAddr;

use tokio::sync::{mpsc, watch};

use crate::{
    Endpoint,
    message::{MandatoryHeaders, Method, Request},
    transaction::{ClientTransaction, TransactionMessage},
    transport::{
        IncomingMessageInfo, IncomingRequest, IncomingResponse, Packet, Transport,
        TransportMessage, mock::MockTransport,
    },
};

use super::fsm::{self};

// ===== client invite tests =====
mod invite;
// ===== client non invite tests =====
mod non_invite;

const PROVISIONAL_1XX_STATUS_CODE: u16 = 100;
const FINAL_NON_2XX_STATUS_CODE: u16 = 301;

struct MockServerTransaction {
    sender: mpsc::Sender<TransactionMessage>,
    request: IncomingRequest,
    endpoint: Endpoint,
}

impl MockServerTransaction {
    pub async fn respond(&self, code: impl TryInto<crate::message::StatusCode>) {
        let response = self.endpoint.create_response(
            &self.request,
            crate::message::StatusCode::try_new(code).unwrap(),
            None,
        );
        let mandatory_headers = MandatoryHeaders::try_from(&response.message.headers).unwrap();
        let transport = TransportMessage {
            packet: Packet::new(response.encoded, response.send_info.target),
            transport: response.send_info.transport,
        };

        let response = IncomingResponse {
            message: response.message,
            info: Box::new(IncomingMessageInfo::new(transport, mandatory_headers)),
        };

        self.sender
            .send(TransactionMessage::Response(response))
            .await
            .unwrap();
    }
}

fn setup_test_send_request(method: Method) -> (Endpoint, Request, (Transport, SocketAddr)) {
    let transport = MockTransport::new_udp();
    let (endpoint, request) =
        super::create_test_endpoint_and_request(method, transport.clone().into());
    let target = (Transport::new(transport), "127.0.0.1:5060".parse().unwrap());

    (endpoint, request.message, target)
}

async fn setup_test_recv_provisional_response(
    method: Method,
) -> (MockServerTransaction, ClientTransaction) {
    let transport = MockTransport::new_udp();
    let (endpoint, request) =
        super::create_test_endpoint_and_request(method, transport.clone().into());
    let target = (Transport::new(transport), "127.0.0.1:5060".parse().unwrap());

    let client = ClientTransaction::send_request(&endpoint, request.message.clone(), Some(target))
        .await
        .unwrap();

    let sender = endpoint
        .transactions()
        .get_entry(client.transaction_key())
        .unwrap();

    let server = MockServerTransaction {
        sender,
        request,
        endpoint,
    };

    (server, client)
}

async fn setup_test_recv_final_response(
    method: Method,
) -> (
    MockServerTransaction,
    ClientTransaction,
    watch::Receiver<fsm::State>,
) {
    let transport = MockTransport::new_udp();
    let (endpoint, request) =
        super::create_test_endpoint_and_request(method, transport.clone().into());
    let target = (Transport::new(transport), "127.0.0.1:5060".parse().unwrap());

    let mut client =
        ClientTransaction::send_request(&endpoint, request.message.clone(), Some(target))
            .await
            .unwrap();

    let sender = endpoint
        .transactions()
        .get_entry(client.transaction_key())
        .unwrap();

    let server = MockServerTransaction {
        sender,
        request,
        endpoint,
    };

    let watch = client.state_machine_mut().subscribe_state();

    (server, client, watch)
}

async fn setup_test_reliable(method: Method) -> (ClientTransaction, MockTransport) {
    let transport = MockTransport::new_tcp();
    let (endpoint, request) =
        super::create_test_endpoint_and_request(method, transport.clone().into());
    let addr = "127.0.0.1:5060".parse().unwrap();
    let target = (Transport::new(transport.clone()), addr);
    let client = ClientTransaction::send_request(&endpoint, request.message.clone(), Some(target))
        .await
        .unwrap();

    (client, transport)
}

async fn setup_test_retransmission(
    method: Method,
) -> (MockServerTransaction, ClientTransaction, MockTransport) {
    let transport = MockTransport::new_udp();
    let transport_clone = transport.clone();
    let (endpoint, request) =
        super::create_test_endpoint_and_request(method, transport.clone().into());
    let target = (
        Transport::new(transport_clone),
        "127.0.0.1:5060".parse().unwrap(),
    );

    let client = ClientTransaction::send_request(&endpoint, request.message.clone(), Some(target))
        .await
        .unwrap();

    assert_eq!(
        client.state(),
        fsm::State::Calling,
        "Transaction state should transition to Calling after sending request"
    );

    let sender = endpoint
        .transactions()
        .get_entry(client.transaction_key())
        .unwrap();

    let server = MockServerTransaction {
        sender,
        request,
        endpoint,
    };

    (server, client, transport)
}
