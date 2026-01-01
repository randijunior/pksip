use std::time::Duration;

use bytes::Bytes;
use tokio::{
    sync::{mpsc, watch},
    time::{self, timeout},
};

use crate::{
    Endpoint,
    endpoint::EndpointBuilder,
    headers,
    message::{MandatoryHeaders, Method, Request, headers::Header},
    transaction::{ClientTransaction, TransactionMessage},
    transport::{
        IncomingMessageInfo, IncomingRequest, Packet, Transport, TransportMessage,
        mock::MockTransport,
    },
};

use super::{ServerTransaction, TransactionState};

mod server;

mod client;

const TEST_FROM_STR: &str = "Alice <sip:alice@localhost>;tag=1928301774";
const TEST_TO_STR: &str = "Bob <sip:bob@localhost>";

#[macro_export]
macro_rules! assert_state_eq {
    ($rx:expr, $expected:expr $(,)?) => {{
        $crate::transaction::tests::wait_state_change(&mut $rx).await;
        assert_eq!(*$rx.borrow(), $expected);
    }};

    ($rx:expr, $expected:expr, $($arg:tt)+) => {{
        crate::transaction::tests::wait_state_change(&mut $rx).await;
        assert_eq!(*$rx.borrow(), $expected, $($arg)+);
    }};
}

async fn wait_state_change(state: &mut watch::Receiver<super::TransactionState>) {
    timeout(Duration::from_secs(1), state.changed())
        .await
        .expect("timeout reached and no state change received")
        .expect("The channel has been closed");
}

fn new_test_request(method: Method, transport: Option<Transport>) -> IncomingRequest {
    let transport = transport.unwrap_or(Transport::new(MockTransport::new_udp()));
    IncomingRequest {
        message: Request::new(method, "sip:localhost".parse().unwrap()),
        info: Box::new(IncomingMessageInfo::new(
            TransportMessage {
                packet: Packet::new(Bytes::new(), transport.local_addr()),
                transport,
            },
            MandatoryHeaders::try_from(&headers! {
                Header::Via(format!(
                    "SIP/2.0/UDP localhost:5060;branch={}",
                    crate::generate_branch(None)
                ).parse().unwrap()),
                Header::From(TEST_FROM_STR.parse().unwrap()),
                Header::To(TEST_TO_STR.parse().unwrap()),
                Header::CallId("a84b4c76e66710@pc33.atlanta.com".parse().unwrap()),
                Header::CSeq(format!("1 {}", method).parse().unwrap()),
            })
            .unwrap(),
        )),
    }
}

fn create_test_endpoint_and_request(
    method: Method,
    transport: Option<MockTransport>,
) -> (Endpoint, IncomingRequest) {
    let request = new_test_request(method, transport.map(Transport::new));
    let endpoint = EndpointBuilder::new()
        .add_transaction(Default::default())
        .build();

    (endpoint, request)
}

fn create_server_transaction(
    method: Method,
    transport: Option<MockTransport>,
) -> (ServerTransaction, watch::Receiver<super::TransactionState>) {
    let request = new_test_request(method, transport.map(Transport::new));
    let endpoint = EndpointBuilder::new()
        .add_transaction(Default::default())
        .build();

    let mut server_tsx = endpoint.create_server_transaction(request).unwrap();
    let state = server_tsx.subscribe_state();

    (server_tsx, state)
}

struct MockTransactionTx {
    sender: mpsc::UnboundedSender<super::TransactionMessage>,
    msg: TransactionMessage,
}

impl MockTransactionTx {
    pub async fn retransmit_to_transaction(&self) {
        self.sender.send(self.msg.clone()).unwrap();
        tokio::task::yield_now().await;
    }

    pub async fn retransmit_n_times(&self, n: usize) {
        for _ in 0..n {
            self.retransmit_to_transaction().await;
        }
    }

    pub async fn send_ack_request(&mut self) {
        self.request_mut().unwrap().message.req_line.method = Method::Ack;
        self.retransmit_to_transaction().await
    }

    pub fn request_mut(&mut self) -> Option<&mut IncomingRequest> {
        if let TransactionMessage::Request(ref mut incoming) = self.msg {
            Some(incoming)
        } else {
            None
        }
    }
}

fn setup_test_server_retransmission(
    method: Method,
) -> (MockTransactionTx, MockTransport, ServerTransaction) {
    let transport = MockTransport::new_udp();
    let transport_clone = transport.clone();

    let (endpoint, request) = create_test_endpoint_and_request(method, transport_clone.into());
    let server = endpoint.create_server_transaction(request.clone()).unwrap();

    let entry = endpoint
        .transactions()
        .get_entry(server.transaction_key())
        .unwrap();

    let sender = MockTransactionTx {
        sender: entry,
        msg: TransactionMessage::Request(request),
    };

    (sender, transport, server)
}

fn setup_test_server_state_reliable(
    method: Method,
) -> (ServerTransaction, watch::Receiver<TransactionState>) {
    create_server_transaction(method, Some(MockTransport::new_tcp()))
}

fn setup_test_server_state_unreliable(
    method: Method,
) -> (ServerTransaction, watch::Receiver<TransactionState>) {
    create_server_transaction(method, Some(MockTransport::new_udp()))
}

fn setup_test_server_receive_ack() -> (
    MockTransactionTx,
    watch::Receiver<TransactionState>,
    ServerTransaction,
) {
    let (sender, _, mut server) = setup_test_server_retransmission(Method::Invite);

    (sender, server.subscribe_state(), server)
}


fn setup_client_state_reliable(method: Method) -> ClientTransaction {
    todo!()
}
