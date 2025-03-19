use async_trait::async_trait;
use sip::{
    endpoint::{Endpoint, EndpointBuilder},
    message::{SipMethod, StatusCode},
    service::SipService,
    transaction::SipTransaction,
    transport::{udp::Udp, IncomingRequest},
};
use std::error::Error;
use tokio::io;

pub struct MyService;

const CODE: StatusCode = StatusCode::Ok;

#[async_trait]
impl SipService for MyService {
    fn name(&self) -> &str {
        "SipUAS"
    }
    async fn on_request(
        &mut self,
        endpoint: &Endpoint,
        req: &mut Option<IncomingRequest>,
    ) -> io::Result<()> {
        let is_options = {
            let req = req.as_ref().unwrap();
            req.is_method(&SipMethod::Options)
        };
        if is_options {
            let req = req.take().unwrap();
            let mut tsx = endpoint.create_uas_tsx(&req);
            let response =
                endpoint.new_response(req, CODE.into()).await?;
            tsx.send_msg(response.into()).await?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // tracing_subscriber::fmt()
    //     .with_max_level(Level::DEBUG)
    //     .with_env_filter("sip=trace")
    //     .init();
    console_subscriber::init();

    let svc = MyService;
    let udp = Udp::bind("0.0.0.0:8080").await?;

    let endpoint = EndpointBuilder::new()
        .with_service(svc)
        .with_transport(udp)
        .build();

    endpoint.run().await?;
    Ok(())
}
