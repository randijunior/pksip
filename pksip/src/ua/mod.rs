use std::collections::HashMap;
use std::sync::Mutex;

pub mod dialog;

use dialog::{Dialog, DialogId};

use crate::message::headers::Contact;
use crate::message::{SipMethod, StatusCode};
use crate::transport::incoming::IncomingRequest;
use crate::{Endpoint, EndpointHandler, Result};

pub struct UserAgent {
    dialogs: Mutex<HashMap<DialogId, Dialog>>,
    endpoint: Endpoint,
}

impl UserAgent {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            dialogs: Default::default(),
        }
    }

    pub fn new_uas_dialog(&self, request: &IncomingRequest, contact: Contact) -> Result<Dialog> {
        let dialog = Dialog::new_uas(self, request, contact)?;

        self.add_dialog(dialog.clone());

        Ok(dialog)
    }

    fn add_dialog(&self, dialog: Dialog) {
        let mut dialogs = self.dialogs.lock().expect("Lock failed");

        dialogs.insert(dialog.id().clone(), dialog);
    }

    fn find_dialog_from_incoming(&self, request: &IncomingRequest) -> Option<Dialog> {
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

#[async_trait::async_trait]
impl EndpointHandler for UserAgent {
    async fn handle(&self, request: IncomingRequest, endpoint: &Endpoint) {}
    // async fn on_incoming_request(
    //     &self,
    //     endpoint: &Endpoint,
    //     request: &mut Option<IncomingRequest>,
    // ) -> Result<()> {
    //     let request_ref = request.as_ref().unwrap();
    //     let method = request_ref.message.method();

    //     let Some(dialog) = self.find_dialog_from_incoming(&request_ref) else {
    //         if method != SipMethod::Ack {
    //             let request = request.take().unwrap();
    //             endpoint
    //                 .respond(&request, StatusCode::CallOrTransactionDoesNotExist, None)
    //                 .await?;
    //         }
    //         return Ok(());
    //     };
    //     let request = request.take().unwrap();

    //     let request_cseq = request.info.mandatory_headers.cseq.cseq;
    //     // Check CSeq.
    //     if request_cseq <= dialog.remote_cseq()
    //         && !matches!(method, SipMethod::Ack | SipMethod::Cancel)
    //     {
    //         endpoint
    //             .respond(
    //                 &request,
    //                 StatusCode::ServerInternalError,
    //                 Some("Invalid Cseq"),
    //             )
    //             .await?;
    //         return Ok(());
    //     }
    //     // Update CSeq.
    //     dialog.set_remote_cseq(request.info.mandatory_headers.cseq.cseq);

    //     let mut request = Some(request);

    //     let usages = dialog.usages().read().await;

    //     for usage in usages.iter() {
    //         usage.on_receive(&mut request).await?;

    //         if request.is_none() {
    //             break;
    //         }
    //     }
    //     drop(usages);

    //     Ok(())
    // }
}
