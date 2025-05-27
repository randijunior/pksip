use std::fmt;

const SIP_INVITE: &[u8] = b"INVITE";
const SIP_CANCEL: &[u8] = b"CANCEL";
const SIP_ACK: &[u8] = b"ACK";
const SIP_BYE: &[u8] = b"BYE";
const SIP_REGISTER: &[u8] = b"REGISTER";
const SIP_OPTIONS: &[u8] = b"OPTIONS";
const SIP_INFO: &[u8] = b"INFO";
const SIP_NOTIFY: &[u8] = b"NOTIFY";
const SIP_SUBSCRIBE: &[u8] = b"SUBSCRIBE";
const SIP_UPDATE: &[u8] = b"UPDATE";
const SIP_REFER: &[u8] = b"REFER";
const SIP_PRACK: &[u8] = b"PRACK";
const SIP_MESSAGE: &[u8] = b"MESSAGE";
const SIP_PUBLISH: &[u8] = b"PUBLISH";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// An SIP Method.
///
/// This enum declares SIP methods as described by RFC3261.
pub enum Method {
    /// SIP INVITE Method.
    Invite,
    /// SIP ACK Method.
    Ack,
    /// SIP BYE Method.
    Bye,
    /// SIP CANCEL Method.
    Cancel,
    /// SIP REGISTER Method.
    Register,
    /// SIP OPTIONS Method.
    Options,
    /// SIP INFO Method.
    Info,
    /// SIP NOTIFY Method.
    Notify,
    /// SIP SUBSCRIBE Method.
    Subscribe,
    /// SIP UPDATE Method.
    Update,
    /// SIP REFER Method.
    Refer,
    /// SIP PRACK Method.
    Prack,
    /// SIP MESSAGE Method.
    Message,
    /// SIP PUBLISH Method.
    Publish,
    /// An unknown SIP method.
    Unknown,
}

impl Method {
    /// Returns the byte representation of a method.
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            Method::Invite => SIP_INVITE,
            Method::Ack => SIP_ACK,
            Method::Bye => SIP_BYE,
            Method::Cancel => SIP_CANCEL,
            Method::Register => SIP_REGISTER,
            Method::Options => SIP_OPTIONS,
            Method::Info => SIP_INFO,
            Method::Notify => SIP_NOTIFY,
            Method::Subscribe => SIP_SUBSCRIBE,
            Method::Update => SIP_UPDATE,
            Method::Refer => SIP_REFER,
            Method::Prack => SIP_PRACK,
            Method::Message => SIP_MESSAGE,
            Method::Publish => SIP_PUBLISH,
            Method::Unknown => b"UNKNOWN-Method",
        }
    }
}

impl From<&[u8]> for Method {
    fn from(value: &[u8]) -> Self {
        match value {
            SIP_INVITE => Method::Invite,
            SIP_CANCEL => Method::Cancel,
            SIP_ACK => Method::Ack,
            SIP_BYE => Method::Bye,
            SIP_REGISTER => Method::Register,
            SIP_OPTIONS => Method::Options,
            SIP_INFO => Method::Info,
            SIP_NOTIFY => Method::Notify,
            SIP_SUBSCRIBE => Method::Subscribe,
            SIP_UPDATE => Method::Update,
            SIP_REFER => Method::Refer,
            SIP_PRACK => Method::Prack,
            SIP_MESSAGE => Method::Message,
            SIP_PUBLISH => Method::Publish,
            _ => Method::Unknown,
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::Invite => write!(f, "INVITE"),
            Method::Ack => write!(f, "ACK"),
            Method::Bye => write!(f, "BYE"),
            Method::Cancel => write!(f, "CANCEL"),
            Method::Register => write!(f, "REGISTER"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Info => write!(f, "INFO"),
            Method::Notify => write!(f, "NOTIFY"),
            Method::Subscribe => write!(f, "SUBSCRIBE"),
            Method::Update => write!(f, "UPDATE"),
            Method::Refer => write!(f, "REFER"),
            Method::Prack => write!(f, "PRACK"),
            Method::Message => write!(f, "MESSAGE"),
            Method::Publish => write!(f, "PUBLISH"),
            Method::Unknown => write!(f, "UNKNOWN-Method"),
        }
    }
}
