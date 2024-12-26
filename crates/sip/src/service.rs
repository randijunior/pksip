use async_trait::async_trait;

use crate::{
    server::SipServer,
    transaction::Transaction,
    transport::{IncomingRequest, IncomingResponse},
};

#[async_trait]
pub trait SipService: Sync + Send + 'static {
    fn name(&self) -> &str;

    async fn on_recv_req(
        &self,
        server: &SipServer,
        inc: &mut Option<IncomingRequest>,
    ) {
    }

    async fn on_recv_res(
        &self,
        server: &SipServer,
        inc: &mut Option<IncomingResponse>,
    ) {
    }

    async fn on_tsx_res(
        &self,
        server: &SipServer,
        res: &mut Option<IncomingResponse>,
        tsx: &Transaction<'_>,
    ) {
    }
}
