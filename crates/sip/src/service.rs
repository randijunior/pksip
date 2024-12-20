use async_trait::async_trait;

use crate::{
    server::SipServer,
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
}
