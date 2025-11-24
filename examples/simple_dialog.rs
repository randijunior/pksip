use std::error::Error;

use async_trait::async_trait;
use pksip::{
    Endpoint, EndpointHandler,
    endpoint::EndpointResponse,
    find_map_header,
    headers::{Header, Headers},
    message::{SipMethod, StatusCode},
    transport::IncomingRequest,
    transaction::TransactionLayer
};
use tracing::Level;

pub struct MyService;

#[async_trait]
impl EndpointHandler for MyService {
    fn name(&self) -> &str {
        "SipUAS"
    }

    async fn on_request(&self, request: &IncomingRequest) -> Option<EndpointResponse> {
        let method = request.message.req_line.method;
        let headers = &request.message.headers;

        let response = if method == SipMethod::Register {
            let mut response = EndpointResponse::stateful(request, StatusCode::Ok, None);

            if let Some(expires) = find_map_header!(headers, Expires) {
                let mut hdrs = Headers::with_capacity(2);
                let expires = *expires;

                hdrs.push(Header::Expires(expires));

                tracing::debug!("{}", expires);
                if expires.as_u32() != 0 {
                    if let Some(contact) = find_map_header!(headers, Contact) {
                        hdrs.push(Header::Contact(contact.clone()));
                    }
                }
                response.message.headers.append(&mut hdrs);
            }
            response
        } else if method != SipMethod::Ack {
            EndpointResponse::stateless(request, StatusCode::NotImplemented, None)
        } else {
            return None;
        };

        Some(response)
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

    let svc = MyService;
    let addr = "127.0.0.1:8089".parse()?;

    let endpoint = Endpoint::builder()
        .add_service(svc)
        .add_transaction(TransactionLayer::default())
        .build();
    endpoint.start_ws(addr).await?;

    let server_addr = format!("ws://{addr}");

    let transport =
        pksip::transport::websocket::WebSocketTransport::connect(&server_addr, 1.0, &endpoint)
            .await?;

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
