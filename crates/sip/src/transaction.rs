use async_trait::async_trait;
use invite_server::ServerInviteTsx;
use non_invite_server::ServerNonInviteTsx;
use tokio::sync::{
    mpsc::{self},
    oneshot,
};

use crate::{
    endpoint::Endpoint,
    headers::CallId,
    internal::ArcStr,
    message::{HostPort, SipMethod, StatusCode},
    transport::{IncomingRequest, MsgBuffer, OutgoingResponse, Transport},
};
use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

pub mod invite_server;
pub mod non_invite_server;

type TsxReceiver = mpsc::Receiver<TsxMsg>;
pub type TsxSender = mpsc::Sender<TsxMsg>;

const T1: Duration = Duration::from_millis(500);
const T2: Duration = Duration::from_secs(4);
const T4: Duration = Duration::from_secs(5);

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
        self.get_state() == TsxState::Proceeding
    }
    pub fn is_trying(&self) -> bool {
        self.get_state() == TsxState::Trying
    }

    pub fn is_completed(&self) -> bool {
        self.get_state() == TsxState::Completed
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
        let headers = req.req_hdrs.as_ref().unwrap();
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

type SharedMsgBuffer = Arc<MsgBuffer>;

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
}

impl Transaction {
    async fn retransmit(&self) -> io::Result<()> {
        if let Some(msg) = self.last_response.as_ref() {
            let buf = msg.buf.as_ref().unwrap();
            self.send_msg(buf).await?;
        }
        Ok(())
    }

    pub fn last_response(&self) -> &Option<OutgoingResponse> {
        &self.last_response
    }

    pub fn last_response_code(&self) -> Option<u32> {
        self.last_response.as_ref().map(|msg| msg.status_code_u32())
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

    async fn send(&mut self, mut response: OutgoingResponse) -> io::Result<()> {
        if let Some(buf) = response.buf {
            self.send_msg(&buf).await?;
            return Ok(());
        }
        let buf = response.into_buffer()?;
        self.send_msg(&buf).await?;

        response.buf = Some(buf.into());
        self.last_response = Some(response.into());
        Ok(())
    }

    pub async fn send_msg(&self, buf: &MsgBuffer) -> io::Result<()> {
        let sended = self.transport.send(buf, self.addr).await?;

        println!("Sended: {sended} bytes");
        Ok(())
    }
}

const BRANCH_RFC3261: &str = "z9hG4bK";

pub enum TsxMsg {
    Request(IncomingRequest),
    Response(OutgoingResponse),
}

impl From<IncomingRequest> for TsxMsg {
    fn from(value: IncomingRequest) -> Self {
        TsxMsg::Request(value)
    }
}

impl From<OutgoingResponse> for TsxMsg {
    fn from(value: OutgoingResponse) -> Self {
        TsxMsg::Response(value)
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

    pub async fn handle_request(
        &self,
        key: &TsxKey,
        request: IncomingRequest,
    ) -> io::Result<Option<IncomingRequest>> {
        if let Some(sender) = self.get(key) {
            println!("TSX FOUND!");
            // Check if is retransmission
            let tsx_msg = TsxMsg::Request(request);
            if let Err(_) = sender.send(tsx_msg).await {
                println!("receiver droped!");
            };
            Ok(None)
        } else {
            println!("TSX NOT FOUND!");
            Ok(Some(request))
        }
    }

    pub(crate) fn spawn_new_tsx(
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
        key: &TsxKey,
        endpoint: &Endpoint,
        request: &mut IncomingRequest,
    ) -> io::Result<(TsxSender, oneshot::Receiver<()>)> {
        let (sender, receiver) = mpsc::channel(100);
        let (tx, rx) = oneshot::channel();
        let addr = request.info.packet().addr();
        let transport = request.info.transport().clone();

        if request.is_method(&SipMethod::Invite) {
            let mut tsx = ServerInviteTsx::new(addr, transport);
            tsx.tx = tx.into();
            let response = endpoint
                .new_response(request, StatusCode::Trying.into())
                .await?;

            sender.send(TsxMsg::Response(response)).await.unwrap();
            // let buf = response.into_buffer()?;

            // response
            //     .info
            //     .transport
            //     .send(&buf, response.info.addr)
            //     .await?;

            self.spawn_new_tsx(tsx, receiver);
        } else {
            let mut tsx = ServerNonInviteTsx::new(addr, transport);
            tsx.tx = tx.into();

            self.spawn_new_tsx(tsx, receiver);
        };

        self.insert(key.clone(), sender.clone());

        Ok((sender, rx))
    }

    pub async fn handle_response(
        &self,
        key: TsxKey,
        response: OutgoingResponse,
    ) -> io::Result<()> {
        if let Some(tsx) = self.get(&key) {
            println!("TSX FOUND RESPONSE!");
            // Check if is retransmission
            let tsx_msg = TsxMsg::Response(response);
            if let Err(_) = tsx.send(tsx_msg).await {
                println!("receiver droped!");
            };
        }

        Ok(())
    }
}
