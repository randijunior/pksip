use std::error::Error;

use async_trait::async_trait;
use pksip::core::service::EndpointService;
use pksip::core::to_take::ToTake;
use pksip::core::SipEndpoint;
use pksip::message::{SipMethod, StatusCode};
use pksip::transaction::Transactions;
use pksip::transport::IncomingRequest;
use pksip::Result;
use tracing::Level;

pub struct MyService;

#[async_trait]
impl EndpointService for MyService {
    fn name(&self) -> &str {
        "SipUAS"
    }

    async fn on_incoming_request(&self, endpoint: &SipEndpoint, request: ToTake<'_, IncomingRequest>) -> Result<()> {
        let request = request.take();

        let method = request.method();
        if method == SipMethod::Options {
            let server_tsx = endpoint.new_server_transaction(&request);
            let mut response = endpoint.new_response(&request, StatusCode::Ok, None);
            server_tsx.respond(&mut response).await?;
        } else if method != SipMethod::Ack {
            endpoint.respond(&request, StatusCode::NotImplemented, None).await?;
        } else {
            // ACK method does not require a response
            tracing::debug!("Received ACK request, no response needed.");
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("pksip=trace")
        // .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(String::from("%H:%M:%S%.3f")))
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .init();

    let svc = MyService;
    let addr = "127.0.0.1:0".parse()?;

    let endpoint = SipEndpoint::builder()
        .with_service(svc)
        .with_transaction(Transactions::default())
        .with_udp(addr)
        .build()
        .await;

    endpoint.run().await?;
    Ok(())
}
