//! Service module.

use crate::core::to_take::ToTake;
use crate::transport::IncomingRequest;
use crate::transport::IncomingResponse;
use crate::Result;
use crate::SipEndpoint;

/// A trait which provides a way to extend the SIP endpoint
/// functionalities.
#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait EndpointService: Sync + Send + 'static {
    /// Returns the service name.
    fn name(&self) -> &str;

    /// Called when an inbound SIP request is received.
    async fn on_incoming_request(
        &self,
        endpoint: &SipEndpoint,
        request: ToTake<'_, IncomingRequest>,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when an inbound SIP response is received.
    async fn on_incoming_response(
        &self,
        endpoint: &SipEndpoint,
        response: &mut Option<IncomingResponse>,
    ) -> Result<()> {
        Ok(())
    }

    // async fn on_transaction_error(&self)
}
