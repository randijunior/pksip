use async_trait::async_trait;
use pksip::{
    endpoint::Endpoint,
    message::{SipMethod, REASON_NOT_IMPLEMENTED},
    transport::IncomingRequest,
    Result, SipService,
};
use std::error::Error;
use tracing::Level;
use tracing_subscriber::fmt::time::ChronoLocal;

pub struct MyService;

#[async_trait]
impl SipService for MyService {
    fn name(&self) -> &str {
        "MyService"
    }
    async fn on_incoming_request(&self, endpoint: &Endpoint, request: &mut Option<IncomingRequest>) -> Result<()> {
        let request = request.take().unwrap();
        if !matches!(request.method(), SipMethod::Ack) {
            endpoint.respond(&request, 501, REASON_NOT_IMPLEMENTED).await?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter("pksip=trace")
        .with_timer(ChronoLocal::new(String::from("%H:%M:%S%.3f")))
        .init();
    // console_subscriber::init();

    let svc = MyService;
    let addr = "127.0.0.1:0".parse()?;

    let endpoint = Endpoint::builder()
        .with_service(svc)
        .with_udp(addr)
        .with_tcp(addr)
        .build()
        .await;

    endpoint.run().await?;
    Ok(())
}
