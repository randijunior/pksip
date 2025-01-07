use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::message::{HostPort, SipMethod};
use crate::transport::{
    IncomingRequest, IncomingResponse, OutgoingInfo, OutgoingResponse,
};

#[derive(PartialEq, Eq, Hash)]
pub struct Rfc2543<'a> {
    pub cseq: u32,
    pub from_tag: Option<&'a str>,
    pub to_tag: Option<&'a str>,
    pub call_id: &'a str,
    pub via_host_port: HostPort<'a>,
    pub method: Option<SipMethod>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Rfc3261<'a> {
    branch: &'a str,
    via_sent_by: HostPort<'a>,
    method: Option<SipMethod>,
}

#[derive(PartialEq, Eq, Hash)]
pub enum TransactionKey<'a> {
    Rfc2543(Rfc2543<'a>),
    Rfc3261(Rfc3261<'a>),
}

impl<'a> From<&IncomingResponse<'a>> for TransactionKey<'a> {
    fn from(value: &IncomingResponse<'a>) -> Self {
        todo!()
    }
}

impl<'a> From<&IncomingRequest<'a>> for TransactionKey<'a> {
    fn from(value: &IncomingRequest<'a>) -> Self {
        todo!()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum TransactionState {
    None,
    Calling,
    Trying,
    Proceeding,
    Completed,
    Confirmed,
    Terminated,
}

#[derive(Clone)]
pub struct ClientTransaction {
    state: TransactionState,
}

#[derive(Clone)]
pub struct NonInviteServerTransaction<'a> {
    state: TransactionState,
    info: OutgoingInfo,
    last_msg: Option<Arc<OutgoingResponse<'a>>>,
}

const T1: Duration = Duration::from_millis(500);
const T2: Duration = Duration::from_secs(4);
const T4: Duration = Duration::from_secs(5);

impl NonInviteServerTransaction<'_> {
    // The state machine is initialized in the "Trying" state and is passed
    // a request other than INVITE or ACK when initialized. This request is
    // passed up to the TU.  Once in the "Trying" state, any further request
    // retransmissions are discarded.  A request is a retransmission if it
    // matches the same server transaction, using the rules specified in
    // Section 17.2.3.
    pub async fn new(method: SipMethod, info: OutgoingInfo) -> Self {
        if !matches!(method, SipMethod::Invite | SipMethod::Ack) {
            panic!("Invalid method for server transaction");
        }

        NonInviteServerTransaction {
            state: TransactionState::Trying,
            info,
            last_msg: None,
        }
    }
}

#[derive(Clone)]
pub enum Transaction<'a> {
    Client(ClientTransaction),
    Server(NonInviteServerTransaction<'a>),
}

impl Transaction<'_> {
    /*
    While in the "Trying" state, if the TU passes a provisional response
    (status codes 100-199) to the server transaction, the server transaction
    MUST enter the "Proceeding" state.  The response MUST be passed to the
    transport layer for transmission.  Any further provisional responses
    that are received from the TU while in the "Proceeding" state MUST
    be passed to the transport layer for transmission.  If a retransmission
    of the request is received while in the "Proceeding" state, the most
    recently sent provisional response MUST be passed to the transport
    layer for retransmission.  If the TU passes a final response (status
    codes 200-699) to the server while in the "Proceeding" state, the
    transaction MUST enter the "Completed" state, and the response MUST
    be passed to the transport layer for transmission.

    When the server transaction enters the "Completed" state, it MUST set
    Timer J to fire in 64*T1 seconds for unreliable transports, and zero
    seconds for reliable transports.  While in the "Completed" state, the
    server transaction MUST pass the final response to the transport
    layer for retransmission whenever a retransmission of the request is
    received.  Any other final responses passed by the TU to the server
    transaction MUST be discarded while in the "Completed" state.  The
    server transaction remains in this state until Timer J fires, at
    which point it MUST transition to the "Terminated" state.

    The server transaction MUST be destroyed the instant it enters the
    "Terminated" state.
    */
    pub fn handle_response(&mut self, resp: &IncomingResponse) {
        if resp.code().is_provisional() {
            match self {
                Transaction::Client(tsx) => todo!(),
                Transaction::Server(tsx) => {
                    tsx.state = TransactionState::Proceeding
                }
            }
        }
    }
}

const BRANCH_RFC3261: &str = "z9hG4bK";

#[derive(Default)]
pub struct Transactions<'a>(
    Mutex<HashMap<TransactionKey<'a>, Transaction<'a>>>,
);

impl Transactions<'_> {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn find_tsx(&self, key: TransactionKey) -> Option<Transaction> {
        self.0.lock().unwrap().get(&key).cloned()
    }
}
