use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::to_take::ToTake;
use crate::dialog::{Dialog, DialogId};
use crate::header::Contact;
use crate::message::{SipMethod, StatusCode};
use crate::transport::IncomingRequest;
use crate::{EndpointService, Result, SipEndpoint};

pub struct UserAgent {
    dialogs: Mutex<HashMap<DialogId, Dialog>>,
    endpoint: SipEndpoint,
}

impl UserAgent {
    pub fn new(endpoint: SipEndpoint) -> Self {
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

    pub fn endpoint(&self) -> &SipEndpoint {
        &self.endpoint
    }
}

#[async_trait::async_trait]
impl EndpointService for UserAgent {
    fn name(&self) -> &str {
        "UserAgent"
    }

    async fn on_incoming_request(
        &self,
        endpoint: &SipEndpoint,
        request: ToTake<'_, IncomingRequest>,
    ) -> Result<()> {
        let method = request.method();

        let Some(dialog) = self.find_dialog_from_incoming(&request) else {
            if method != SipMethod::Ack {
                let request = request.take();
                endpoint
                    .respond(&request, StatusCode::CallOrTransactionDoesNotExist, None)
                    .await?;
            }
            return Ok(());
        };

        let request = request.take();

        let request_cseq = request.request_headers.cseq.cseq;
        // Check CSeq.
        if request_cseq <= dialog.remote_cseq()
            && !matches!(method, SipMethod::Ack | SipMethod::Cancel)
        {
            // let response = request.create_response(
            //     StatusCode::ServerInternalError,
            //     Some("Invalid Cseq".into()),
            // )?;
            endpoint
                .respond(
                    &request,
                    StatusCode::ServerInternalError,
                    Some("Invalid Cseq".into()),
                )
                .await?;
            return Ok(());
        }
        // Update CSeq.
        dialog.set_remote_cseq(request.request_headers.cseq.cseq);

        let mut request = Some(request);

        let usages = dialog.usages().read().await;

        for usage in usages.iter() {
            usage.on_receive(ToTake::new(&mut request)).await?;

            if request.is_none() {
                break;
            }
        }
        drop(usages);
        Ok(())
    }
}
