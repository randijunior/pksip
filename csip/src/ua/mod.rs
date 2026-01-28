use std::collections::HashMap;
use std::sync::Mutex;

pub(crate) mod inv;

use tokio::sync::mpsc;

use crate::dialog::{Dialog, DialogId, DialogMessage};

use crate::message::headers::Contact;
use crate::transport::incoming::IncomingRequest;
use crate::{Endpoint, Method, Result};

pub struct UserAgent {
    dialogs: Mutex<HashMap<DialogId, mpsc::Sender<DialogMessage>>>,
    endpoint: Endpoint,
}

impl UserAgent {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            dialogs: Default::default(),
        }
    }

    pub async fn on_received_request(&self, request: IncomingRequest) -> Option<IncomingRequest> {
        if request.req_line.method == Method::Cancel {
            return Some(request);
        }
        let Some(sender) = self.find_dialog_from_incoming(&request) else { 
            return Some(request);
        };
        let _res = sender.send(DialogMessage::Request(request)).await;
       None
    }

    pub fn new_uas_dialog(&self, request: IncomingRequest, contact: Contact) -> Result<Dialog> {
        let dialog = Dialog::create_uas(self, request, contact)?;

        Ok(dialog)
    }

    pub(crate) fn add_dialog(&self, dialog_id: DialogId, dialog: mpsc::Sender<DialogMessage>) {
        let mut dialogs = self.dialogs.lock().expect("Lock failed");

        dialogs.insert(dialog_id, dialog);
    }

    fn find_dialog_from_incoming(&self, request: &IncomingRequest) -> Option<mpsc::Sender<DialogMessage>> {
        let Some(dialog_id) = DialogId::from_incoming_request(request) else {
            return None;
        };
        let dialogs = self.dialogs.lock().expect("Lock failed");

        dialogs.get(&dialog_id).cloned()
    }

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
}