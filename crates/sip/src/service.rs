use std::io;

use async_trait::async_trait;

use crate::transport::{IncomingRequest, IncomingResponse};

use crate::endpoint::Endpoint;

#[async_trait]
pub trait SipService: Sync + Send + 'static {
    fn name(&self) -> &str;

    async fn on_request(
        &self,
        endpt: &Endpoint,
        req: &mut Option<IncomingRequest>
    ) -> io::Result<()> {
        Ok(())
    }

    async fn on_response(
        &self,
        endpt: &Endpoint,
        response: &mut Option<IncomingResponse>,
    ) -> io::Result<()> {
        Ok(())
    }
}