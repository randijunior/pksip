use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use pksip::message::{SipMethod, SipResponse, StatusCode};
use pksip::transaction::TransactionManager;
use pksip::transport::incoming::IncomingRequest;
use pksip::{Endpoint, EndpointHandler};
use tokio::time;
use tracing::Level;

pub struct UasHandler;

#[async_trait]
impl EndpointHandler for UasHandler {
    async fn handle(&self, request: IncomingRequest, endpoint: &Endpoint) {
        if request.req_line.method == SipMethod::Options {
            let uas = endpoint.new_server_transaction(request).unwrap();
            let response = SipResponse::builder()
                .status(StatusCode::Ok)
                .reason("Ok")
                .build();

            uas.respond_with_final(response).await.unwrap();

            return;
        }
        if request.req_line.method != SipMethod::Ack {
            let response = SipResponse::builder()
                .status(StatusCode::NotImplemented)
                .build();

            let _res = endpoint.respond(&request, response).await;

            return;
        }

        tracing::debug!("Received ACK request, no response needed.");
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("pksip=trace")
        .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .init();

    let svc = UasHandler;
    let addr = "127.0.0.1:0".parse()?;

    let endpoint = Endpoint::builder()
        .with_handler(svc)
        .with_transaction(TransactionManager::new())
        .build();

    endpoint.start_tcp_transport(addr).await?;
    endpoint.start_udp_transport(addr).await?;
    endpoint.start_ws_transport(addr).await?;

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
