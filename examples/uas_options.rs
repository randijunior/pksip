use std::{error::Error, time::Duration};

use async_trait::async_trait;
use pksip::{
    Endpoint, EndpointHandler,
    endpoint::EndpointResponse,
    message::{SipMethod, StatusCode},
    transport::IncomingRequest,
};
use tokio::time;
use tracing::Level;

pub struct MyService;

#[async_trait]
impl EndpointHandler for MyService {
    fn name(&self) -> &str {
        "SipUAS"
    }

    async fn on_request(&self, request: &IncomingRequest) -> Option<EndpointResponse> {
        match request.message.req_line.method {
            SipMethod::Options => {
                let response = EndpointResponse::stateful(request, StatusCode::Ok, None);

                Some(response)
            }
            method if method != SipMethod::Ack => {
                let response = EndpointResponse::stateless(request, StatusCode::NotImplemented, None);

                Some(response)
            }
            _ => {
                tracing::debug!("Received ACK request, no response needed.");
                None
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("pksip=trace")
        // .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(String::from("%H:%M:%S%.3f"
        // )))
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .init();

    let svc = MyService;
    let addr = "127.0.0.1:0".parse()?;

    let endpoint = Endpoint::builder()
        .add_service(svc)
        .add_transaction(Default::default())
        .build();

    endpoint.start_tcp(addr).await?;
    endpoint.start_udp(addr).await?;
    endpoint.start_ws(addr).await?;

    loop {
        tokio::select! {
            _ = time::sleep(Duration::from_secs(1)) => {
            }
            _ = tokio::signal::ctrl_c() => {
            println!();
            break;
        }
        }
    }
    Ok(())
}
