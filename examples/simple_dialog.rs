use std::error::Error;

use async_trait::async_trait;
use pksip::{
    Endpoint, EndpointHandler,
    endpoint::{self},
    find_map_header,
    message::{
        SipMethod, StatusCode,
        headers::{Header, Headers},
    },
    transaction::{ServerTransaction, TransactionManager},
    transport::IncomingRequest,
};
use tracing::Level;

pub struct SimpleDialogHandler;

#[async_trait]
impl EndpointHandler for SimpleDialogHandler {
    async fn handle(&self, request: IncomingRequest, endpoint: &Endpoint) -> pksip::Result<()> {
        let method = request.message.req_line.method;
        let headers = &request.message.headers;

        if method == SipMethod::Register {
            let new_header = if let Some(expires) = find_map_header!(headers, Expires) {
                let mut hdrs = Headers::with_capacity(2);
                let expires = *expires;

                hdrs.push(Header::Expires(expires));

                tracing::debug!("{}", expires);
                if expires.as_u32() != 0 {
                    if let Some(contact) = find_map_header!(headers, Contact) {
                        hdrs.push(Header::Contact(contact.clone()));
                    }
                }
                Some(hdrs)
            } else {
                None
            };
            let uas = endpoint.create_server_transaction(request)?;
            uas.send_final_response(StatusCode::Ok, None, new_header, None)
                .await?;
        } else if method != SipMethod::Ack {
            endpoint
                .respond_stateless(&request, StatusCode::NotImplemented, None)
                .await?;
        } else {
            return Ok(());
        };
        Ok(())
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("pksip=debug,simple_dialog=trace")
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            String::from("%H:%M:%S%.3f"),
        ))
        // .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .init();

    let svc = SimpleDialogHandler;
    let addr = "127.0.0.1:8089".parse()?;

    let endpoint = Endpoint::builder()
        .with_handler(svc)
        .with_transaction(TransactionManager::default())
        .build();
    endpoint.start_ws_transport(addr).await?;

    let server_addr = format!("ws://{addr}");

    let transport =
        pksip::transport::ws::WebSocketTransport::connect(&server_addr, 1.0, &endpoint).await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // tokio::select! {
    //     _ = endpoint.run() => {
    //         println!("received done signal!");
    //     }
    //     _ = tokio::signal::ctrl_c() => {
    //         println!();
    //     }
    // };
}
