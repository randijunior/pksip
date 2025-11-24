use std::error::Error;

use async_trait::async_trait;
use pksip::{
    Endpoint, EndpointHandler,
    endpoint::EndpointResponse,
    message::{SipMethod, StatusCode},
    transport::IncomingRequest,
};
use tracing::Level;
use tracing_subscriber::fmt::time::ChronoLocal;

pub struct MyService;

#[async_trait]
impl EndpointHandler for MyService {
    fn name(&self) -> &str {
        "MyService"
    }

    async fn on_request(&self, request: &IncomingRequest) -> Option<EndpointResponse> {
        if request.message.req_line.method != SipMethod::Ack {
            let response = EndpointResponse::stateless(request, StatusCode::NotImplemented, None);
            Some(response)
        } else {
            None
        }
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

    let endpoint = Endpoint::builder().add_service(svc).build();

    endpoint.start_ws(addr).await?;
    endpoint.start_udp(addr).await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
