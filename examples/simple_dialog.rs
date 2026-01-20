use std::error::Error;

use async_trait::async_trait;
use pksip::message::headers::{Header, Headers};
use pksip::message::{SipMethod, StatusCode};
use pksip::transaction::TransactionManager;
use pksip::transport::incoming::IncomingRequest;
use pksip::{Endpoint, EndpointHandler, find_map_header};
use tracing::Level;

pub struct SimpleDialogHandler;

#[async_trait]
impl EndpointHandler for SimpleDialogHandler {
    async fn handle(&self, incoming: IncomingRequest, endpoint: &Endpoint) -> pksip::Result<()> {
        let method = incoming.request.req_line.method;
        let headers = &incoming.request.headers;

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
            let uas = endpoint.create_server_transaction(incoming)?;
            uas.send_final_response(StatusCode::Ok, None, new_header, None)
                .await?;
        } else if method != SipMethod::Ack {
            endpoint
                .respond_stateless(&incoming, StatusCode::NotImplemented, None)
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
