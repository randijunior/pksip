use std::ops;

use crate::message::{MandatoryHeaders, SipRequest, SipResponse};

/// This type represents an received SIP request.
#[derive(Clone)]
pub struct IncomingRequest {
    /// The SIP message.
    pub request: SipRequest,
    /// Incoming message info.
    pub incoming_info: Box<IncomingInfo>,
}

impl ops::Deref for IncomingRequest {
    type Target = SipRequest;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

/// This type represents an received SIP response.
#[derive(Clone)]
pub struct IncomingResponse {
    /// The SIP message.
    pub response: SipResponse,
    /// Incoming message info.
    pub incoming_info: Box<IncomingInfo>,
}

impl ops::Deref for IncomingResponse {
    type Target = SipResponse;
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

/// Incoming message info.
#[derive(Clone)]
pub struct IncomingInfo {
    /// The mandatory headers extracted from the message.
    pub mandatory_headers: MandatoryHeaders,
    /// The received transport packet.
    pub transport: super::TransportMessage,
}
