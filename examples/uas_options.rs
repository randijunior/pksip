use async_trait::async_trait;
use pksip::{
    endpoint::{Builder, Endpoint},
    message::{Method, REASON_NOT_IMPLEMENTED, REASON_OK},
    service::SipService,
    transaction::TransactionLayer,
    transport::IncomingRequest,
    Result,
};
use std::error::Error;
use tracing::Level;


pub struct MyService;

#[async_trait]
impl SipService for MyService {
    fn name(&self) -> &str {
        "SipUAS"
    }
    async fn on_incoming_request(&self, endpoint: &Endpoint, request: &mut IncomingRequest) -> Result<bool> {
        match request.method() {
            Method::Options => {
                let tsx = endpoint.new_uas_tsx(request);
                let mut response = endpoint.new_response(request, 200, REASON_OK);
                tsx.respond(&mut response).await?;

                Ok(true)
            }
            &method if method != Method::Ack => {
                endpoint.respond(request, 501, REASON_NOT_IMPLEMENTED).await?;

                Ok(true)
            }
            _ => Ok(false),
        }
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

    let endpoint = Builder::new()
        .with_service(svc)
        .with_transaction_layer(TransactionLayer::default())
        .with_udp(addr)
        .build()
        .await;

    endpoint.run().await?;
    Ok(())
}
