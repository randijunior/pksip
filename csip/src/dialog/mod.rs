use tokio::sync::mpsc;

use crate::Endpoint;
use crate::error::{DialogError, Result};
use crate::message::headers::{CallId, Contact, From, Header, Headers, To};
use crate::message::{Method, Params, ReasonPhrase, Scheme, StatusCode, Uri};
use crate::transaction::Role;
use crate::transport::incoming::IncomingRequest;
use crate::ua::UserAgent;

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

/// Returns `true` if this method can establish a dialog
const fn can_establish_a_dialog(method: &Method) -> bool {
        matches!(method, Method::Invite)
}

/// Represents a SIP Dialog.
pub struct Dialog {
    endpoint: Endpoint,
    id: DialogId,
    state: DialogState,
    remote_cseq: u32,
    local_seq_num: Option<u32>,
    from: From,
    to: To,
    contact: Contact,
    secure: bool,
    route_set: Vec<RouteSet>,
    role: Role,
    usages: Vec<Box<dyn DialogUsage>>,
    receiver: mpsc::Receiver<DialogMessage>,
}


impl Dialog {
    pub fn create_uas(ua: &UserAgent, request: IncomingRequest, contact: Contact) -> Result<Self> {
        if !can_establish_a_dialog(&request.req_line.method) {
            return Err(DialogError::InvalidMethod.into());
        }
        let request_headers = &request.incoming_info.mandatory_headers;
        let all_headers = &request.request.headers;

        let Some(local_tag) = request_headers.to.tag().clone() else {
            return Err(DialogError::MissingTagInToHeader.into());
        };

        let mut to = request_headers.to.clone();
        let from = request_headers.from.clone();

        let remote_cseq = request_headers.cseq.cseq;
        let local_seq_num = None;

        let route_set = RouteSet::from_headers(all_headers);
        let secure = request.incoming_info.transport.transport.is_secure()
            && request.request.req_line.uri.scheme == Scheme::Sips;

        to.set_tag(Some(crate::generate_tag_n(16)));

        let dialog_id = DialogId {
            call_id: request_headers.call_id.clone(),
            remote_tag: from.tag().clone().unwrap_or_default(),
            local_tag,
        };

        let (sender, receiver) = mpsc::channel(10);

        ua.add_dialog(dialog_id.clone(), sender);

        let transaction = ua.endpoint().new_server_transaction(request);

        let dialog = Self {
            endpoint: ua.endpoint().clone(),
            id: dialog_id,
            state: DialogState::Early,
            remote_cseq,
            local_seq_num,
            from,
            to,
            contact,
            secure,
            route_set,
            role: Role::UAS,
            usages: Vec::new(),
            receiver,
        };

        Ok(dialog)
    }

    pub async fn receive(&mut self, request: IncomingRequest) -> Result<()> {
        // Check CSeq.
        let request_cseq = request.incoming_info.mandatory_headers.cseq.cseq;

        if request_cseq <= self.remote_cseq
            && !matches!(request.req_line.method, Method::Ack | Method::Cancel)
        {
            let st_text = ReasonPhrase::from("Invalid Cseq");
            self.endpoint
                .respond(&request, StatusCode::ServerInternalError, Some(st_text))
                .await?;
            return Ok(());
        }
        self.remote_cseq = request_cseq;
        let mut request = Some(request);

        for usage in self.usages.iter() {
            usage.on_receive(&mut request).await?;

            if request.is_none() {
                break;
            }
        }

        Ok(())
    }

    pub async fn register_usage<U>(&mut self, usage: U)
    where
        U: DialogUsage,
    {
        self.usages.push(Box::new(usage));
    }
}

pub enum DialogMessage {
    Request(IncomingRequest),
}

#[async_trait::async_trait]
pub trait DialogUsage: Sync + Send + 'static {
    async fn on_receive(&self, request: &mut Option<IncomingRequest>) -> Result<()>;
}

enum DialogState {
    // Initial state, before any request is sent or received
    Early,
    // Established
    Established,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogId {
    call_id: CallId,
    pub local_tag: String,
    remote_tag: String,
}

impl DialogId {
    pub fn from_incoming_request(request: &IncomingRequest) -> Option<Self> {
        let call_id = request.incoming_info.mandatory_headers.call_id.clone();

        let local_tag = match request.incoming_info.mandatory_headers.to.tag() {
            Some(tag) => tag.clone(),
            None => return None,
        };

        let remote_tag = match request.incoming_info.mandatory_headers.from.tag() {
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
    params: Option<Params>,
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
