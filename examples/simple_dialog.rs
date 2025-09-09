use std::error::Error;

use async_trait::async_trait;
use pksip::core::service::EndpointService;
use pksip::core::to_take::ToTake;
use pksip::core::SipEndpoint;
use pksip::find_map_header;
use pksip::header::Header;
use pksip::header::Headers;
use pksip::message::SipMethod;
use pksip::message::StatusCode;
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
        let headers = &request.msg.headers;

        if request.msg.req_line.method == SipMethod::Register {
            let mut response = endpoint.new_response(&request, StatusCode::Ok, None);

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
                response.append_headers(&mut hdrs);
            }

            let server_tsx = endpoint.new_server_transaction(&request);
            server_tsx.respond(&mut response).await?;
        } else if request.msg.req_line.method != SipMethod::Ack {
            endpoint.respond(&request, StatusCode::NotImplemented, None).await?;
        }

        Ok(())

        // if request.msg.req_line.method == SipMethod::Invite {
        //     // TODO: create dialog and inv session
        // }

        // Ok(())
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("pksip=debug,simple_dialog=trace")
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(String::from(
            "%H:%M:%S%.3f",
        )))
        // .with_timer(tracing_subscriber::fmt::time::SystemTime)
        .init();

    let svc = MyService;
    let addr = "127.0.0.1:0".parse()?;

    let endpoint = SipEndpoint::builder()
        .with_service(svc)
        .with_transaction(Transactions::default())
        .with_tcp(addr)
        .with_udp(addr)
        .build()
        .await;

    tokio::select! {
        _ = endpoint.run() => {
            println!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };
    Ok(())
}
