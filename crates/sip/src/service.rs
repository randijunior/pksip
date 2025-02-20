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

    async fn on_response(
        &self,
        endpt: &Endpoint,
        response: &mut Option<IncomingResponse>,
    ) -> io::Result<()> {
        Ok(())
    }
}

pub struct Request {
    pub endpoint: Endpoint,
    pub msg: Option<IncomingRequest>,
    pub tsx: TsxSender,
}

impl Request {
    pub async fn reply(&mut self, st_line: StatusLine) -> io::Result<()> {
        let mut msg = self.msg.take().unwrap();
        let response = self.endpoint.new_response(&mut msg, st_line).await?;

        let _res = self.tsx.send(response.into()).await;

        Ok(())
    }
}
