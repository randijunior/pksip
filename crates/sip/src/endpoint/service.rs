use async_trait::async_trait;

use crate::{transaction::Transaction, transport::{IncomingRequest, IncomingResponse}};

use super::Endpoint;

#[async_trait]
pub trait SipService: Sync + Send + 'static {
    fn name(&self) -> &str;

    async fn on_request(
        &self,
        endpt: &Endpoint,
        inc: &mut Option<IncomingRequest>,
    ) {
    }

    async fn on_response(
        &self,
        endpt: &Endpoint,
        inc: &mut Option<IncomingResponse>,
    ) {
    }

    async fn on_transaction_response(
        &self,
        endpt: &Endpoint,
        res: &mut Option<IncomingResponse>,
        tsx: &Transaction<'_>,
    ) {
    }
}
