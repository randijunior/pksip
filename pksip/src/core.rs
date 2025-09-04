pub mod endpoint;

pub use endpoint::EndpointBuilder;
pub use endpoint::SipEndpoint;
pub use service::EndpointService;

pub mod service;

pub mod to_take;
pub mod ua;
