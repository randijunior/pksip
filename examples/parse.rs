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

#[async_trait]
impl SipService for MyService {
    fn name(&self) -> &str {
        "MyService"
    }
    async fn on_request(
        &self,
        endpt: &Endpoint,
        req: &mut Option<IncomingRequest>,
    ) -> io::Result<()> {
        if !req.as_ref().unwrap().is_method(&SipMethod::Ack) {
            let msg = req.take().unwrap();
            endpt
                .respond(msg, StatusCode::NotImplemented.into())
                .await?;
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

    let svc = MyService;
    let udp = Udp::bind("0.0.0.0:5060").await?;

    let endpoint = EndpointBuilder::new()
        .with_service(svc)
        .with_transport(udp)
        .build();

    endpoint.run().await?;
    Ok(())
}
