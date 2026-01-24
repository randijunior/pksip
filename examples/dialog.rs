use std::error::Error;

use async_trait::async_trait;
use pksip::message::headers::{Header, Headers};
use pksip::message::{SipResponse, SipMethod, StatusCode, StatusLine};
use pksip::transaction::TransactionManager;
use pksip::transport::incoming::IncomingRequest;
use pksip::{Endpoint, EndpointHandler, find_map_header};
use tracing::Level;

pub struct SimpleDialogHandler;

#[async_trait]
impl EndpointHandler for SimpleDialogHandler {
    async fn handle(&self, request: IncomingRequest, endpoint: &Endpoint) {
        if request.req_line.method == SipMethod::Register {
            let headers = &request.headers;
            let mut hdrs = Headers::new();
            if let Some(expires) = find_map_header!(headers, Expires) {
                let expires = *expires;

                hdrs.push(Header::Expires(expires));

                tracing::debug!("{}", expires);
                if expires.as_u32() != 0 {
                    if let Some(contact) = find_map_header!(headers, Contact) {
                        hdrs.push(Header::Contact(contact.clone()));
                    }
                }
            }
            let uas = endpoint.new_server_transaction(request).unwrap();

            let status_line = StatusLine::new(StatusCode::Ok, "Ok".into());
            let response = SipResponse::with_headers(status_line, hdrs);

            uas.send_final(response).await.unwrap();
        } else if request.req_line.method != SipMethod::Ack {
            let response = SipResponse::builder()
                .status(StatusCode::NotImplemented)
                .build();

            endpoint.respond(&request, response).await.unwrap();
        }
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
        .init();

    let svc = SimpleDialogHandler;
    let addr = "127.0.0.1:8089".parse()?;

    let endpoint = Endpoint::builder()
        .with_handler(svc)
        .with_transaction(TransactionManager::default())
        .build();
    endpoint.start_ws_transport(addr).await?;

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
