use tokio::sync::Mutex;

use crate::headers::Header;
use crate::transaction::inv_client::InvClientTransaction;
use crate::transaction::inv_server::InvServerTransaction;
use crate::transaction::ServerTsx;
use crate::transport::{IncomingRequest, OutgoingRequest, OutgoingResponse};
use crate::ua::Dialog;

pub enum Session {
    UAC(UacSession),
    UAS(UasSession),
}

impl Session {
    pub fn new_uas(request: IncomingRequest<'static>, dialog: Dialog) -> Self {
        Self::UAS(UasSession::new(dialog, request))
    }
}

pub enum SessionState {
    Early,
    Confirmed,
    Terminated,
}

pub struct UasSession {
    session: DialogSession,
    transaction: InvServerTransaction,
    request: IncomingRequest<'static>,
}

impl UasSession {
    pub fn new(dialog: Dialog, mut request: IncomingRequest<'static>) -> Self {
        let Some(ServerTsx::Invite(transaction)) = request.transaction.take() else {
            panic!("Expected a UAS invite transaction");
        };
        let session = DialogSession {
            dialog,
            state: Mutex::new(SessionState::Early),
        };

        Self {
            session,
            transaction,
            request,
        }
    }

    pub fn create_response<'a>(&'a self, status_code: i32, reason_phrase: &'a str) -> OutgoingResponse<'a> {
        let mut response = self
            .session
            .dialog
            .endpoint()
            .new_response(&self.request, status_code, reason_phrase);

        if status_code != 100 {
            let to = response
                .headers_mut()
                .iter_mut()
                .find_map(|h| if let Header::To(to) = h { Some(to) } else { None })
                .expect("To header not found");

            // Set the To tag from the dialog.
            to.set_tag(Some(&self.session.dialog.id().local_tag));

            // response.set_dialog(&self.dialog);
        }

        response
    }
}

pub struct UacSession {
    session: DialogSession,
    transaction: InvClientTransaction,
    request: OutgoingRequest<'static>,
}

pub struct DialogSession {
    pub dialog: Dialog,
    pub state: Mutex<SessionState>,
}
