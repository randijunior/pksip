//! Service module.
//! 

use crate::{
    transport::{IncomingRequest, IncomingResponse},
    Endpoint, Result,
};

/// A trait which provides a way to extend the SIP endpoint functionalities.
#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait SipService: Sync + Send + 'static {
    /// Returns the service name.
    fn name(&self) -> &str;

    /// Called when an inbound SIP request is received.
    async fn on_incoming_request(&self, endpoint: &Endpoint, request: &mut Option<IncomingRequest>) -> Result<()> {
        Ok(())
    }

    /// Called when an inbound SIP response is received.
    async fn on_incoming_response(&self, endpoint: &Endpoint, response: &mut Option<IncomingResponse>) -> Result<()> {
        Ok(())
    }

    // async fn on_transaction_error(&self)
}
