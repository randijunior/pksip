use async_trait::async_trait;
use sip::{
    endpoint::{Endpoint, EndpointBuilder},
    message::{SipMethod, StatusCode},
    service::SipService,
    transport::{udp::Udp, IncomingRequest},
};
use std::error::Error;
use tokio::io;
use tracing::Level;

pub struct MyService;

const CODE: StatusCode = StatusCode::NotImplemented;

#[async_trait]
impl SipService for MyService {
    fn name(&self) -> &str {
        "MyService"
    }
    async fn on_request(
        &mut self,
        endpt: &Endpoint,
        req: &mut Option<IncomingRequest>,
    ) -> io::Result<()> {
        let is_ack = {
            let req = req.as_ref().unwrap();
            req.is_method(&SipMethod::Ack)
        };
        if !is_ack {
            let msg = req.take().unwrap();
            endpt.respond(msg, CODE.into()).await?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_env_filter("sip=trace")
        .init();
    // console_subscriber::init();

    let svc = MyService;
    let udp = Udp::bind("0.0.0.0:8080").await?;

    let endpoint = EndpointBuilder::new()
        .with_service(svc)
        .with_transport(udp)
        .build();

    endpoint.run().await?;
    Ok(())
}
