use std::sync::{atomic::AtomicU32, Arc, Mutex};

use crate::{
    headers::{CallId, Contact, From, To},
    message::{Params, SipMethod, Uri},
    transaction::Role,
    transport::IncomingRequest,
    ua::Ua,
    Endpoint,
};

/**
 * Example of SIP Dialog establishment and termination (INVITE):
 *
 * UAC (Caller)                 UAS (Receiver)
 *     |--- INVITE ----------->|    // Request to establish a session
 *     |<--- 180 Ringing ------|    // Indicates ringing (early dialog)
 *     |<--- 200 OK -----------|    // Session accepted → Dialog created (confirmed)
 *     |--- ACK --------------->|   // Confirms receipt of 200 OK → Dialog active
 *     |--- BYE --------------->|   // Terminates the session
 *     |<--- 200 OK -----------|    // Confirms termination → Dialog terminated
 */

struct Inner {
    endpoint: Endpoint,
    // Unique identifier for the dialog
    id: DialogID,
    // Current state of the dialog
    state: Mutex<DialogState>,
    // Remote sequence number (last CSeq received)
    remote_seq_num: u32,
    // Local sequence number (next CSeq to be sent)
    local_seq_num: Option<AtomicU32>,
    // From header
    from: From<'static>,
    // To header
    to: To<'static>,
    // Contact header for sending requests to the remote UA
    contact: Contact<'static>,
    // Whether the dialog was established over a secure transport (TLS)
    secure: bool,
    // Ordered list of URIs used for routing (from Record-Route)
    route_set: Vec<RouteSet>,
    role: Role,
}

fn generate_random_str() -> String {
    todo!("Implement a function to generate a random string for tags")
}

/// Represents a SIP Dialog.
#[derive(Clone)]
pub struct Dialog {
    inner: Arc<Inner>,
}

impl Dialog {
    pub fn id(&self) -> &DialogID {
        &self.inner.id
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.inner.endpoint
    }

    pub fn create_uas(ua: &Ua, request: &mut IncomingRequest<'static>, contact: &str) -> crate::Result<Dialog> {
        if request.request_headers.to.tag().is_none() {
            todo!("Error: To header must have a tag for UAS dialog creation");
        }
        let method = request.method();
        if !Self::is_supported_dialog_method(method) {
            todo!("Error: InvalidMethodForDialog");
        }
        // Local party.
        let mut to = request.to().clone().into_owned();
        // Remote party.
        let from = request.from().clone().into_owned();
        // The remote sequence number MUST be set to the value of the sequence
        // number in the CSeq header field of the request.
        let remote_seq_num = request.request_headers.cseq.cseq;
        // The local sequence number MUST be empty.
        let local_seq_num: Option<AtomicU32> = None;
        // Parse the contact header.
        let contact = match Contact::from_str(contact) {
            Ok(contact) => contact.into_owned(),
            Err(err) => return Err(err),
        };

        // TODO: The route set MUST be set to the list of URIs in the Record-Route
        // header field from the request, taken in order and preserving all URI parameters.
        let route_set: Vec<RouteSet> = request
            .request
            .headers
            .iter()
            .filter_map(|h| match h {
                crate::headers::Header::RecordRoute(route) => Some(route),
                _ => None,
            })
            .map(|route| RouteSet {
                uri: route.addr.uri.clone().into_owned(),
                params: route.params.as_ref().map(|p| p.clone().into_owned()),
            })
            .collect();

        to.set_tag_onwed(generate_random_str().into());

        let dialog_id = DialogID {
            call_id: request.call_id().clone().into_owned(),
            remote_tag: from.tag().clone().unwrap_or_default().into_owned(),
            local_tag: to.tag().clone().unwrap().into_owned(),
        };

        let dialog = Dialog {
            inner: Arc::new(Inner {
                endpoint: ua.endpoint().clone(),
                id: dialog_id,
                state: Mutex::new(DialogState::Initial),
                remote_seq_num,
                local_seq_num,
                from,
                to,
                contact,
                // TODO: Check If the request arrived over TLS, and the Request-URI contained a SIPS
                // URI
                secure: false,
                route_set,
                role: Role::UAS,
            }),
        };

        if let Some(tsx) = request.transaction.as_ref() {
            // Associate this dialog to the transaction?.
            tsx.set_dialog(dialog.clone());
        } else {
            let transaction = ua.endpoint().new_uas_inv_tsx(request);
            transaction.set_dialog(dialog.clone());
            request.set_tsx_inv(transaction);
        };

        ua.register_dialog(dialog.clone());

        Ok(dialog)
    }

    fn is_supported_dialog_method(method: &SipMethod) -> bool {
        use SipMethod::*;
        matches!(method, Invite | Subscribe | Refer | Update | Notify)
    }
}

struct RouteSet {
    uri: Uri<'static>,
    params: Option<Params<'static>>,
}

/// Unique identifier of a SIP dialog (Call-ID + From tag + To tag).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogID {
    call_id: CallId<'static>,
    // From tag
    // TODO: Change to Arc<str>
    pub local_tag: String,
    // To tag
    // TODO: Change to Arc<str>
    remote_tag: String,
}

enum DialogState {
    // Initial state, before any request is sent or received
    Initial,
    // Established
    Established,
}
