use async_trait::async_trait;
use transaction_layer::Transaction;
use transport_layer::transport::{RxRequest, RxResponse};

use crate::server::SipServer;

#[async_trait]
pub trait SipService: Sync + Send + 'static {
    fn name(&self) -> &str;

    async fn on_recv_req(
        &self,
        server: &SipServer,
        inc: &mut Option<RxRequest>,
    ) {
    }

    async fn on_recv_res(
        &self,
        server: &SipServer,
        inc: &mut Option<RxResponse>,
    ) {
    }

    async fn on_tsx_res(
        &self,
        server: &SipServer,
        res: &mut Option<RxResponse>,
        tsx: &Transaction<'_>,
    ) {
    }
}
