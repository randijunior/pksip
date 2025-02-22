use std::io;

use async_trait::async_trait;

use crate::message::StatusLine;
use crate::transaction::TsxSender;
use crate::transport::{IncomingRequest, IncomingResponse};

use crate::endpoint::Endpoint;

#[async_trait]
pub trait SipService: Sync + Send + 'static {
    fn name(&self) -> &str;

    async fn on_request(&self, req: &mut Request) -> io::Result<()> {
        Ok(())
    }

    async fn on_recv_response(
        &self,
        endpt: &Endpoint,
        response: &mut Option<IncomingResponse>,
    ) -> io::Result<()> {
        Ok(())
    }
}

pub struct Request<'a> {
    pub endpoint: &'a Endpoint,
    pub msg: Option<IncomingRequest>,
}
