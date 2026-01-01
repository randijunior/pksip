use std::{error::Error, time::Duration};

use async_trait::async_trait;
use pksip::{Endpoint, EndpointHandler, message::Method, transport::IncomingRequest};
use tokio::time;
use tracing::Level;

pub struct UasOptionsHandler;

#[async_trait]
impl EndpointHandler for UasOptionsHandler {
    async fn handle(&self, request: IncomingRequest, endpoint: &Endpoint) -> pksip::Result<()> {
        if request.req_line.method == Method::Options {
            let server_tx = endpoint.create_server_transaction(request)?;

            server_tx.respond_with_final_code(200).await?;

            return Ok(());
        }
        if request.req_line.method != Method::Ack {
            endpoint.send_response(&request, 501, None).await?;

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
        .add_handler(svc)
        .add_transaction(Default::default())
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
