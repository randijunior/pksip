use std::{time::Duration};

use bytes::Bytes;
use tokio::{
    sync::{watch},
    time::{timeout},
};

use crate::{
    Endpoint,
    endpoint::EndpointBuilder,
    headers,
    message::{MandatoryHeaders, Method, Request, headers::{Header, MaxForwards}},
    transaction::TransactionMessage,
    transport::{
        IncomingMessageInfo, IncomingRequest,  Packet, Transport,
        TransportMessage, mock::MockTransport,
    },
};

use super::fsm::{self};

mod server;

mod client;

const STATUS_CODE_100_TRYING: u16 = 100;
const STATUS_CODE_180_RINGING: u16 = 180;
const STATUS_CODE_202_ACCEPTED: u16 = 202;
const STATUS_CODE_301_MOVED_PERMANENTLY: u16 = 301;
const STATUS_CODE_404_NOT_FOUND: u16 = 404;
const STATUS_CODE_504_SERVER_TIMEOUT: u16 = 504;
const STATUS_CODE_603_DECLINE: u16 = 603;

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

async fn wait_state_change(state: &mut watch::Receiver<fsm::State>) {
    timeout(Duration::from_secs(1), state.changed())
        .await
        .expect("timeout reached and no state change received")
        .expect("The channel has been closed");
}

fn create_test_request(method: Method, transport: Option<Transport>) -> IncomingRequest {
    let transport = transport.unwrap_or(Transport::new(MockTransport::new_udp()));
    let headers = headers! {
        Header::Via(format!(
            "SIP/2.0/UDP localhost:5060;branch={}",
            crate::generate_branch(None)
        ).parse().unwrap()),
        Header::From(TEST_FROM_STR.parse().unwrap()),
        Header::To(TEST_TO_STR.parse().unwrap()),
        Header::CallId("a84b4c76e66710@pc33.atlanta.com".parse().unwrap()),
        Header::CSeq(format!("1 {}", method).parse().unwrap()),
        Header::MaxForwards(MaxForwards::new(70))
    };
    let mandatory_headers =  MandatoryHeaders::try_from(&headers)
    .unwrap();
    IncomingRequest {
        message: Request::with_headers(method, "sip:localhost".parse().unwrap(), headers),
        info: Box::new(IncomingMessageInfo::new(
            TransportMessage {
                packet: Packet::new(Bytes::new(), transport.local_addr()),
                transport,
            },
           mandatory_headers,
        )),
    }
}

fn create_test_endpoint_and_request(
    method: Method,
    transport: Option<MockTransport>,
) -> (Endpoint, IncomingRequest) {
    let request = create_test_request(method, transport.map(Transport::new));
    let endpoint = EndpointBuilder::new()
        .add_transaction(Default::default())
        .build();

    (endpoint, request)
}


