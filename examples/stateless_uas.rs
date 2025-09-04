use std::error::Error;

use async_trait::async_trait;
use pksip::core::to_take::ToTake;
use pksip::core::SipEndpoint;
use pksip::message::SipMethod;
use pksip::message::StatusCode;
use pksip::transport::IncomingRequest;
use pksip::EndpointService;
use pksip::Result;
use tracing::Level;
use tracing_subscriber::fmt::time::ChronoLocal;

pub struct MyService;

#[async_trait]
impl EndpointService for MyService {
    fn name(&self) -> &str {
        "MyService"
    }

    async fn on_incoming_request(&self, endpoint: &SipEndpoint, request: ToTake<'_, IncomingRequest>) -> Result<()> {
        let request = request.take();

        if !matches!(request.method(), SipMethod::Ack) {
            endpoint.respond(&request, StatusCode::NotImplemented, None).await?;
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
    // console_subscriber::init();

    let svc = MyService;
    let addr = "127.0.0.1:0".parse()?;

    let endpoint = SipEndpoint::builder()
        .with_service(svc)
        .with_udp(addr)
        .with_tcp(addr)
        .build()
        .await;

    endpoint.run().await?;
    Ok(())
}
