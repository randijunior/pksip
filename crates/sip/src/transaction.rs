use crate::{
    endpoint::Endpoint,
    headers::CallId,
    internal::ArcStr,
    message::{HostPort, SipMethod, StatusCode},
    transport::{
        IncomingRequest, IncomingResponse, MsgBuffer, OutGoingRequest,
        OutgoingResponse, Transport,
    },
};
use async_trait::async_trait;
use server_inv::TsxUasInv;
use server_non_inv::TsxUas;
use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::sync::{
    mpsc::{self},
    oneshot,
};

pub mod client_non_inv;
pub mod server_inv;
pub mod server_non_inv;

type TsxReceiver = mpsc::Receiver<TsxMsg>;
pub type TsxSender = mpsc::Sender<TsxMsg>;

const T1: Duration = Duration::from_millis(500);
const T2: Duration = Duration::from_secs(4);
const T4: Duration = Duration::from_secs(5);

#[async_trait]
pub trait SipTransaction: Sync + Send + 'static {
    async fn recv_msg(&mut self, msg: TsxMsg) -> io::Result<()>;
}

pub struct Transaction {
    state: TsxStateMachine,
    addr: SocketAddr,
    transport: Transport,
    last_response: Option<OutgoingResponse>,
    tx: Option<oneshot::Sender<()>>,
    retransmit_count: Arc<AtomicUsize>,
}

impl Transaction {
    pub fn set_tsx_timers(&mut self) {
        todo!()
    }
    pub fn retransmission_count(&self) -> u32 {
        self.retransmit_count.load(Ordering::SeqCst) as u32
    }
    async fn retransmit(&mut self) -> io::Result<()> {
        if let Some(msg) = self.last_response.as_ref() {
            let buf = msg.buf.as_ref().unwrap();
            self.send_buf(buf).await?;
            self.retransmit_count.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }

    pub fn last_response(&self) -> &Option<OutgoingResponse> {
        &self.last_response
    }

    pub fn last_response_code(&self) -> Option<StatusCode> {
        self.last_response.as_ref().map(|msg| msg.status_code())
    }

    fn do_terminate(&mut self, time: Duration) {
        let tx = self.tx.take();
        if self.reliable() {
            self.state.terminated();
            if let Some(tx) = tx {
                tx.send(()).unwrap();
            }
            return;
        }
        let state = self.state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(time).await;
            state.terminated();
            if let Some(tx) = tx {
                tx.send(()).unwrap();
            }
        });
    }

    fn reliable(&self) -> bool {
        self.transport.reliable()
    }

    fn get_state(&self) -> TsxState {
        self.state.get_state()
    }

    async fn send(&mut self, mut res: OutgoingResponse) -> io::Result<()> {
        if let Some(buf) = res.buf {
            self.send_buf(&buf).await?;
            return Ok(());
        }
        let buf = res.into_buffer()?;
        self.send_buf(&buf).await?;

        res.buf = Some(buf.into());
        self.last_response = Some(res);
        Ok(())
    }

    pub async fn send_buf(&self, buf: &MsgBuffer) -> io::Result<()> {
        let sended = self.transport.send(buf, self.addr).await?;

        println!("Sended: {sended} bytes");
        Ok(())
    }
}

const BRANCH_RFC3261: &str = "z9hG4bK";

#[derive(Clone)]
pub struct TsxStateMachine(Arc<Mutex<TsxState>>);

impl TsxStateMachine {
    pub fn new(state: TsxState) -> Self {
        Self(Arc::new(Mutex::new(state)))
    }
    pub fn set_state(&self, new_state: TsxState) {
        let mut state = self.0.lock().unwrap();
        *state = new_state;
    }

    pub fn terminated(&self) {
        self.set_state(TsxState::Terminated);
    }

    pub fn trying(&self) {
        self.set_state(TsxState::Trying);
    }

    pub fn proceeding(&self) {
        self.set_state(TsxState::Proceeding);
    }

    pub fn completed(&self) {
        self.set_state(TsxState::Completed);
    }

    pub fn confirmed(&self) {
        self.set_state(TsxState::Confirmed);
    }

    pub fn get_state(&self) -> TsxState {
        *self.0.lock().unwrap()
    }

    pub fn is_proceeding(&self) -> bool {
        self.get_state().is_proceeding()
    }
    pub fn is_trying(&self) -> bool {
        self.get_state().is_trying()
    }

    pub fn is_completed(&self) -> bool {
        self.get_state().is_completed()
    }
    pub fn is_terminated(&self) -> bool {
        self.get_state().is_terminated()
    }
    pub fn is_confirmed(&self) -> bool {
        self.get_state().is_confirmed()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsxState {
    Trying,
    Proceeding,
    Completed,
    Confirmed,
    Terminated,
}

impl TsxState {
    pub fn is_proceeding(&self) -> bool {
        *self == TsxState::Proceeding
    }
    pub fn is_trying(&self) -> bool {
        *self == TsxState::Trying
    }

    pub fn is_completed(&self) -> bool {
        *self == TsxState::Completed
    }

    pub fn is_terminated(&self) -> bool {
        *self == TsxState::Terminated
    }

    pub fn is_confirmed(&self) -> bool {
        *self == TsxState::Confirmed
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TsxKey {
    Rfc2543(Rfc2543),
    Rfc3261(Rfc3261),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Rfc2543 {
    pub cseq: u32,
    pub from_tag: Option<ArcStr>,
    pub to_tag: Option<ArcStr>,
    pub call_id: CallId,
    pub via_host_port: HostPort,
    pub method: Option<SipMethod>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Rfc3261 {
    branch: ArcStr,
    via_sent_by: HostPort,
    method: Option<SipMethod>,
    cseq: u32,
}

impl TryFrom<&IncomingRequest> for TsxKey {
    type Error = TsxKeyError;

    fn try_from(req: &IncomingRequest) -> Result<Self, Self::Error> {
        let headers = req.msg.req_headers.as_ref().unwrap();
        let via = &headers.via[0];

        if let Some(branch) = &via.branch {
            // RFC 3261
            let key = Rfc3261 {
                branch: branch.clone(),
                via_sent_by: via.sent_by.clone(),
                // Ack not use
                method: Some(req.msg.method()),
                cseq: headers.cseq.cseq,
            };

            Ok(TsxKey::Rfc3261(key))
        } else {
            todo!("RFC 2543")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TsxKeyError;

pub enum TsxMsg {
    UasRequest(IncomingRequest),
    UasResponse(OutgoingResponse),
    UacRequest(OutGoingRequest),
    UacResponse(IncomingResponse),
}

impl TsxMsg {
    pub fn uas_request(&self) -> Option<&IncomingRequest> {
        if let TsxMsg::UasRequest(req) = self {
            Some(req)
        } else {
            None
        }
    }

    pub fn tsx_key(&self) -> Option<&TsxKey> {
        match self {
            TsxMsg::UasRequest(incoming_request) => {
                incoming_request.tsx_key.as_ref()
            }
            TsxMsg::UasResponse(outgoing_response) => todo!(),
            TsxMsg::UacRequest(out_going_request) => todo!(),
            TsxMsg::UacResponse(incoming_response) => todo!(),
        }
    }

    pub fn uas_request_mut(&mut self) -> Option<&mut IncomingRequest> {
        if let TsxMsg::UasRequest(req) = self {
            Some(req)
        } else {
            None
        }
    }
}

impl From<IncomingRequest> for TsxMsg {
    fn from(value: IncomingRequest) -> Self {
        TsxMsg::UasRequest(value)
    }
}

impl From<OutgoingResponse> for TsxMsg {
    fn from(value: OutgoingResponse) -> Self {
        TsxMsg::UasResponse(value)
    }
}

#[derive(Default)]
pub struct TransactionLayer(Mutex<HashMap<TsxKey, TsxSender>>);

impl TransactionLayer {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn remove(&self, key: &TsxKey) -> Option<TsxSender> {
        self.0.lock().unwrap().remove(key)
    }

    pub fn get(&self, key: &TsxKey) -> Option<TsxSender> {
        self.0.lock().unwrap().get(key).cloned()
    }

    pub fn insert(&self, key: TsxKey, tsx: TsxSender) {
        self.0.lock().unwrap().insert(key, tsx);
    }

    pub async fn handle_message(
        &self,
        message: TsxMsg,
    ) -> io::Result<Option<TsxMsg>> {
        let key = message.tsx_key().unwrap();
        if let Some(sender) = self.get(key) {
            println!("TSX FOUND!");
            // Check if is retransmission
            if (sender.send(message).await).is_err() {
                println!("receiver droped!");
            };
            Ok(None)
        } else {
            Ok(Some(message))
        }
    }

    pub(crate) fn tsx_recv_msg(
        &self,
        mut tsx: impl SipTransaction,
        mut rx: TsxReceiver,
    ) {
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                tsx.recv_msg(msg).await.unwrap()
            }
        });
    }

    pub async fn create_uas_tsx(
        &self,
        endpoint: &Endpoint,
        request: &mut IncomingRequest,
    ) -> io::Result<oneshot::Receiver<()>> {
        let (sender, receiver) = mpsc::channel(100);
        let (tx, rx) = oneshot::channel();

        if request.is_method(&SipMethod::Invite) {
            let mut tsx = TsxUasInv::new(request, endpoint).await?;
            tsx.tx = Some(tx);
            self.tsx_recv_msg(tsx, receiver);
        } else {
            let mut tsx = TsxUas::new(request);
            tsx.tx = Some(tx);
            self.tsx_recv_msg(tsx, receiver);
        };
        let key = request.tsx_key.as_ref().unwrap();
        self.insert(key.clone(), sender);

        Ok(rx)
    }
}

#[cfg(test)]
pub(crate) mod mock {
    use std::time::SystemTime;

    use crate::{
        headers::{CSeq, Headers},
        message::{RequestLine, SipRequest, SipResponse, SipUri},
        transport::{
            udp::mock::MockUdpTransport, IncomingInfo, OutgoingInfo, Packet,
            RequestHeaders,
        },
    };

    use super::*;
    pub fn response(c: StatusCode) -> TsxMsg {
        let from = "sip:alice@127.0.0.1:5060".parse().unwrap();
        let to = "sip:bob@127.0.0.1:5060".parse().unwrap();
        let cseq = CSeq {
            cseq: 1,
            method: SipMethod::Options,
        };
        let callid = CallId::new("bs9ki9iqbee8k5kal8mpqb");
        let hdrs = Box::new(RequestHeaders {
            via: vec![],
            from,
            to,
            callid,
            cseq,
        });
        let transport = Transport::new(MockUdpTransport);
        let info = OutgoingInfo {
            addr: transport.addr(),
            transport,
        };
        let msg = SipResponse::new(c.into(), Headers::new(), None);
        let response = OutgoingResponse {
            hdrs,
            msg,
            info,
            buf: None,
        };

        response.into()
    }

    pub fn request(m: SipMethod) -> TsxMsg {
        let target = "sip:bob@127.0.0.1:5060".parse().unwrap();
        let from = "sip:alice@127.0.0.1:5060".parse().unwrap();
        let to = "sip:bob@127.0.0.1:5060".parse().unwrap();
        let via = "SIP/2.0/UDP 127.0.0.1:5060".parse().unwrap();
        let SipUri::Uri(uri) = target else {
            unreachable!()
        };
        let cseq = CSeq { cseq: 1, method: m };
        let callid = CallId::new("bs9ki9iqbee8k5kal8mpqb");
        let hdrs = Box::new(RequestHeaders {
            via: vec![via],
            from,
            to,
            callid,
            cseq,
        });
        let transport = Transport::new(MockUdpTransport);
        let packet = Packet {
            payload: "".as_bytes().into(),
            addr: transport.addr(),
            time: SystemTime::now(),
        };

        let info = IncomingInfo::new(packet, transport);
        let req_line = RequestLine { method: m, uri };
        let req = SipRequest::new(req_line, Headers::new(), None, Some(hdrs));
        let incoming = IncomingRequest::new(req, info);

        incoming.into()
    }
}
