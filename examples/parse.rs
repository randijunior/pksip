use async_trait::async_trait;
use sip::{
    endpoint::EndpointBuilder,
    service::{Request, SipService},
    transport::udp::Udp,
};
use std::error::Error;
use tokio::io;

pub struct MyService;

#[async_trait]
impl SipService for MyService {
    fn name(&self) -> &str {
        "MyService"
    }
    async fn on_request(&self, req: &mut Request) -> io::Result<()> {
        // ...
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let svc = MyService;
    let udp = Udp::bind("127.0.0.1:5060").await?;

    let endpoint = EndpointBuilder::new()
        .with_service(svc)
        .with_transport(udp)
        .build();

    if let Err(err) = endpoint.run().await {
        println!("{err}");
    }
    Ok(())
}
