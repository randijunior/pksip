use async_trait::async_trait;

use crate::{endpoint::Endpoint, transport::IncomingMessage};

#[async_trait]
pub trait SipService: Sync + Send + 'static  {
    fn name(&self) -> &str;
    async fn on_recv_req(&self, endpoint: &Endpoint, inc: &mut Option<IncomingMessage>);
}