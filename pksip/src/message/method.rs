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
pub enum SipMethod {
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

impl SipMethod {
    /// Returns the byte representation of a method.
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            SipMethod::Invite => SIP_INVITE,
            SipMethod::Ack => SIP_ACK,
            SipMethod::Bye => SIP_BYE,
            SipMethod::Cancel => SIP_CANCEL,
            SipMethod::Register => SIP_REGISTER,
            SipMethod::Options => SIP_OPTIONS,
            SipMethod::Info => SIP_INFO,
            SipMethod::Notify => SIP_NOTIFY,
            SipMethod::Subscribe => SIP_SUBSCRIBE,
            SipMethod::Update => SIP_UPDATE,
            SipMethod::Refer => SIP_REFER,
            SipMethod::Prack => SIP_PRACK,
            SipMethod::Message => SIP_MESSAGE,
            SipMethod::Publish => SIP_PUBLISH,
            SipMethod::Unknown => b"UNKNOWN-SipMethod",
        }
    }
}

impl From<&[u8]> for SipMethod {
    fn from(value: &[u8]) -> Self {
        match value {
            SIP_INVITE => SipMethod::Invite,
            SIP_CANCEL => SipMethod::Cancel,
            SIP_ACK => SipMethod::Ack,
            SIP_BYE => SipMethod::Bye,
            SIP_REGISTER => SipMethod::Register,
            SIP_OPTIONS => SipMethod::Options,
            SIP_INFO => SipMethod::Info,
            SIP_NOTIFY => SipMethod::Notify,
            SIP_SUBSCRIBE => SipMethod::Subscribe,
            SIP_UPDATE => SipMethod::Update,
            SIP_REFER => SipMethod::Refer,
            SIP_PRACK => SipMethod::Prack,
            SIP_MESSAGE => SipMethod::Message,
            SIP_PUBLISH => SipMethod::Publish,
            _ => SipMethod::Unknown,
        }
    }
}

impl fmt::Display for SipMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SipMethod::Invite => write!(f, "INVITE"),
            SipMethod::Ack => write!(f, "ACK"),
            SipMethod::Bye => write!(f, "BYE"),
            SipMethod::Cancel => write!(f, "CANCEL"),
            SipMethod::Register => write!(f, "REGISTER"),
            SipMethod::Options => write!(f, "OPTIONS"),
            SipMethod::Info => write!(f, "INFO"),
            SipMethod::Notify => write!(f, "NOTIFY"),
            SipMethod::Subscribe => write!(f, "SUBSCRIBE"),
            SipMethod::Update => write!(f, "UPDATE"),
            SipMethod::Refer => write!(f, "REFER"),
            SipMethod::Prack => write!(f, "PRACK"),
            SipMethod::Message => write!(f, "MESSAGE"),
            SipMethod::Publish => write!(f, "PUBLISH"),
            SipMethod::Unknown => write!(f, "UNKNOWN-SipMethod"),
        }
    }
}
