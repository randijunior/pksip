use std::error::Error;

use async_trait::async_trait;
use pksip::message::{SipMethod, StatusCode};
use pksip::transport::incoming::IncomingRequest;
use pksip::{Endpoint, EndpointHandler, Result};
use tracing::Level;
use tracing_subscriber::fmt::time::ChronoLocal;

pub struct StatelessUasHandler;

const CODE: StatusCode = StatusCode::NotImplemented;

#[async_trait]
impl EndpointHandler for StatelessUasHandler {
    async fn handle(&self, incoming: IncomingRequest, endpoint: &Endpoint) -> Result<()> {
        if incoming.request.req_line.method != SipMethod::Ack {
            endpoint.respond_stateless(&incoming, CODE, None).await?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("pksip=trace")
        .with_timer(ChronoLocal::new(String::from("%H:%M:%S%.3f")))
        .init();

    let svc = StatelessUasHandler;
    let addr = "127.0.0.1:0".parse()?;

    let endpoint = Endpoint::builder().with_handler(svc).build();

    endpoint.start_ws_transport(addr).await?;
    endpoint.start_udp_transport(addr).await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
