use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use pksip::message::{SipMethod, StatusCode};
use pksip::transport::incoming::IncomingRequest;
use pksip::{Endpoint, EndpointHandler};
use tokio::time;
use tracing::Level;

pub struct UasOptionsHandler;

#[async_trait]
impl EndpointHandler for UasOptionsHandler {
    async fn handle(&self, incoming: IncomingRequest, endpoint: &Endpoint) -> pksip::Result<()> {
        if incoming.request.req_line.method == SipMethod::Options {
            let uas = endpoint.create_server_transaction(incoming)?;

            uas.respond_final_code(StatusCode::Ok).await?;

            return Ok(());
        }
        if incoming.request.req_line.method != SipMethod::Ack {
            endpoint
                .respond_stateless(&incoming, StatusCode::NotImplemented, None)
                .await?;

            return Ok(());
        }

        tracing::debug!("Received ACK request, no response needed.");
        Ok(())
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

    let svc = UasOptionsHandler;
    let addr = "127.0.0.1:0".parse()?;

    let endpoint = Endpoint::builder()
        .with_handler(svc)
        .with_transaction(Default::default())
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
