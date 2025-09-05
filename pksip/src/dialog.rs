use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;

use crate::core::to_take::ToTake;
use crate::core::ua::UserAgent;
use crate::error::{DialogError, Result};
use crate::header::{CallId, Contact, From, Header, Headers, To};
use crate::message::{Parameters, Uri};
use crate::transaction::Role;
use crate::transport::IncomingRequest;
use crate::SipEndpoint;

type Usages = RwLock<Vec<Box<dyn DialogUsage>>>;

/**
 * Example of SIP Dialog establishment and termination
 * (INVITE):
 *
 * UAC (Caller)                 UAS (Receiver)
 *     |--- INVITE ----------->|    // Request to establish a session
 *     |<--- 180 Ringing ------|    // Indicates ringing (early dialog)
 *     |<--- 200 OK -----------|    // InvSession accepted → Dialog created (confirmed)
 *     |--- ACK --------------->|   // Confirms receipt of 200 OK → Dialog active
 *     |--- BYE --------------->|   // Terminates the session
 *     |<--- 200 OK -----------|    // Confirms termination → Dialog terminated
 */

struct Inner {
    /// SipEndpoint associated with the dialog
    endpoint: SipEndpoint,
    // Unique identifier for the dialog
    id: DialogId,
    // Current state of the dialog
    state: Mutex<DialogState>,
    // Remote sequence number (last CSeq received)
    remote_seq_num: AtomicU32,
    // Local sequence number (next CSeq to be sent)
    local_seq_num: Option<AtomicU32>,
    // From header
    from: From,
    // To header
    to: To,
    // Contact header for sending requests to the remote UserAgent
    contact: Contact,
    // Whether the dialog was established over a secure transport (TLS)
    secure: bool,
    // Ordered list of URIs used for routing (from Record-Route)
    route_set: Vec<RouteSet>,
    /// Role of the dialog (UAC or UAS)
    role: Role,
    /// Dialog usages.
    usages: Usages,
}

/// Represents a SIP Dialog.
#[derive(Clone)]
pub struct Dialog {
    inner: Arc<Inner>,
}

impl Dialog {
    pub fn new_uas(ua: &UserAgent, request: &IncomingRequest, contact: Contact) -> Result<Dialog> {
        if request.to().tag().is_none() {
            return Err(DialogError::MissingTagInToHeader.into());
        }
        let method = request.method();
        if !method.can_establish_a_dialog() {
            return Err(DialogError::InvalidMethod(method).into());
        }

        let request_headers = &request.request_headers;
        let all_headers = &request.msg.headers;

        let mut to = request_headers.to.clone();
        let from = request_headers.from.clone();

        let remote_seq_num = request_headers.cseq.cseq.into();
        let local_seq_num = None;

        let route_set = RouteSet::from_headers(all_headers);
        let secure = request.is_secure();

        to.set_tag(crate::generate_random_str().into());

        let dialog_id = DialogId {
            call_id: request.call_id().clone(),
            remote_tag: from.tag().clone().unwrap_or_default(),
            local_tag: to.tag().clone().unwrap(),
        };

        let inner = Inner {
            endpoint: ua.endpoint().clone(),
            id: dialog_id.clone(),
            state: DialogState::Initial.into(),
            remote_seq_num,
            local_seq_num,
            from,
            to,
            contact,
            secure,
            route_set,
            role: Role::UAS,
            usages: RwLock::new(Vec::new()),
        };

        let dialog = Dialog {
            inner: Arc::new(inner),
        };

        Ok(dialog)
    }

    pub fn id(&self) -> &DialogId {
        &self.inner.id
    }

    pub fn endpoint(&self) -> &SipEndpoint {
        &self.inner.endpoint
    }

    pub async fn register_usage<U>(&self, usage: U)
    where
        U: DialogUsage,
    {
        let mut usages = self.inner.usages.write().await;
        usages.push(Box::new(usage));
    }

    pub fn set_remote_cseq(&self, cseq: u32) {
        self.inner.remote_seq_num.store(cseq, Ordering::SeqCst);
    }

    pub fn remote_cseq(&self) -> u32 {
        self.inner.remote_seq_num.load(Ordering::Relaxed)
    }

    pub fn usages(&self) -> &Usages {
        &self.inner.usages
    }
}

#[async_trait::async_trait]
pub trait DialogUsage: Sync + Send + 'static {
    async fn on_receive(&self, request: ToTake<'_, IncomingRequest>) -> Result<()>;
}

enum DialogState {
    // Initial state, before any request is sent or received
    Initial,
    // Established
    Established,
}

/// Unique identifier of a SIP dialog (Call-ID + From tag +
/// To tag).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogId {
    call_id: CallId,
    pub local_tag: String,
    remote_tag: Arc<str>,
}

impl DialogId {
    pub fn from_incoming_request(request: &IncomingRequest) -> Option<Self> {
        let call_id = request.call_id().clone();

        let local_tag = match request.to().tag() {
            Some(tag) => tag.clone(),
            None => return None,
        };

        let remote_tag = match request.from().tag() {
            Some(tag) => tag.clone(),
            None => return None,
        };

        Some(Self {
            call_id,
            local_tag,
            remote_tag,
        })
    }
}

struct RouteSet {
    uri: Uri,
    params: Option<Parameters>,
}

impl RouteSet {
    pub fn from_headers(headers: &Headers) -> Vec<RouteSet> {
        headers
            .iter()
            .filter_map(|header| {
                if let Header::RecordRoute(route) = header {
                    Some(RouteSet {
                        uri: route.addr.uri.clone(),
                        params: route.params.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
