use async_trait::async_trait;

use crate::{transaction::Transaction, transport::{RxRequest, RxResponse}};

use super::Endpoint;

#[async_trait]
pub trait SipService: Sync + Send + 'static {
    fn name(&self) -> &str;

    async fn on_recv_req(
        &self,
        endpt: &Endpoint,
        inc: &mut Option<RxRequest>,
    ) {
    }

    async fn on_recv_res(
        &self,
        endpt: &Endpoint,
        inc: &mut Option<RxResponse>,
    ) {
    }

    async fn on_tsx_res(
        &self,
        endpt: &Endpoint,
        res: &mut Option<RxResponse>,
        tsx: &Transaction<'_>,
    ) {
    }
}
