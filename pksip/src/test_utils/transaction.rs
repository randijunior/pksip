use std::time::Duration;

use bytes::Bytes;
use tokio::{
    sync::{mpsc, watch},
    time::timeout,
};

use crate::{
    Endpoint,
    endpoint::EndpointBuilder,
    headers,
    message::{
        MandatoryHeaders, Request, SipMethod,
        headers::{Header, MaxForwards},
    },
    transaction::{T1, T2, TransactionMessage, fsm},
    transport::{
        IncomingMessageInfo, IncomingRequest, IncomingResponse, Packet, Transport, TransportMessage,
    },
};

const FROM_HDR_STR: &str = "Alice <sip:alice@localhost>;tag=1928301774";
const TO_HDR_STR: &str = "Bob <sip:bob@localhost>";
const CALLID_STR: &str = "a84b4c76e66710@pc33.atlanta.com";

/// Asserts that the last state received in the [`watch::Receiver<State>`] are equal to the expected.
#[macro_export]
macro_rules! assert_state_eq {
    ($watcher:expr, $state:expr $(,)?) => {{
        $crate::assert_state_eq!($watcher, $state,)
    }};

    ($watcher:expr, $state:expr, $($arg:tt)+) => {{
        $crate::test_utils::transaction::wait_state_change(&mut $watcher).await;
        assert_eq!(*$watcher.borrow(), $state, $($arg)+);
    }};
}

pub async fn wait_state_change(state: &mut watch::Receiver<fsm::State>) {
    timeout(Duration::from_secs(1), state.changed())
        .await
        .expect("timeout reached and no state change received")
        .expect("The channel has been closed");
}

pub fn create_test_endpoint() -> Endpoint {
    EndpointBuilder::new()
        .with_transaction(Default::default())
        .build()
}

pub fn create_test_request(method: SipMethod, transport: Transport) -> IncomingRequest {
    let headers = headers! {
        Header::Via(format!(
            "SIP/2.0/UDP localhost:5060;branch={}",
            crate::generate_branch(None)
        ).parse().unwrap()),
        Header::From(FROM_HDR_STR.parse().unwrap()),
        Header::To(TO_HDR_STR.parse().unwrap()),
        Header::CallId(CALLID_STR.into()),
        Header::CSeq(format!("1 {}", method).parse().unwrap()),
        Header::MaxForwards(MaxForwards::new(70))
    };
    let mandatory_headers = MandatoryHeaders::try_from(&headers).unwrap();
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

#[allow(unused_variables)]
pub trait TestContext: Sized {
    fn setup(method: SipMethod) -> Self {
        unimplemented!()
    }
    async fn setup_async(method: SipMethod) -> Self {
        unimplemented!()
    }
}

pub struct MockServerTransaction {
    pub sender: mpsc::Sender<TransactionMessage>,
    pub request: IncomingRequest,
    pub endpoint: Endpoint,
}

impl MockServerTransaction {
    pub async fn respond(&self, code: crate::message::StatusCode) {
        let mandatory_headers = self.request.info.mandatory_headers.clone();
        let response = self.endpoint.create_response(&self.request, code, None);
        let packet = Packet::new(response.encoded, response.send_info.target);

        let transport = TransportMessage {
            packet,
            transport: response.send_info.transport,
        };
        let info = IncomingMessageInfo {
            transport,
            mandatory_headers,
        };

        let response = IncomingResponse::new(response.message, info);

        self.sender
            .send(TransactionMessage::Response(response))
            .await
            .unwrap();
    }
}

pub struct MockClientTransaction {
    pub sender: mpsc::Sender<TransactionMessage>,
    pub request: IncomingRequest,
}

impl MockClientTransaction {
    pub async fn retransmit_request(&self) {
        self.sender
            .send(TransactionMessage::Request(self.request.clone()))
            .await
            .unwrap();
        tokio::task::yield_now().await;
    }

    pub async fn retransmit_n_times(&self, n: usize) {
        for _ in 0..n {
            self.retransmit_request().await;
        }
    }

    pub async fn send_ack_request(&mut self) {
        let mut request = self.request.clone();
        request.message.req_line.method = SipMethod::Ack;
        self.sender
            .send(TransactionMessage::Request(request))
            .await
            .unwrap();
        tokio::task::yield_now().await;
    }
}

pub struct TestRetransmissionTimer {
    interval: Duration,
}

impl TestRetransmissionTimer {
    pub fn new() -> Self {
        Self { interval: T1 }
    }

    pub async fn wait_for_retransmissions(&mut self, n: usize) {
        for _ in 0..n {
            tokio::time::sleep(self.interval).await;
            self.interval = std::cmp::min(self.interval * 2, T2);
            tokio::task::yield_now().await;
        }
    }
}
