use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// An SIP SipMethod.
///
/// This enum declares SIP methods as described by RFC3261 and Others.
pub enum SipMethod {
    /// SIP INVITE SipMethod.
    Invite,
    /// SIP ACK SipMethod.
    Ack,
    /// SIP BYE SipMethod.
    Bye,
    /// SIP CANCEL SipMethod.
    Cancel,
    /// SIP REGISTER SipMethod.
    Register,
    /// SIP OPTIONS SipMethod.
    Options,
    /// SIP INFO SipMethod.
    Info,
    /// SIP NOTIFY SipMethod.
    Notify,
    /// SIP SUBSCRIBE SipMethod.
    Subscribe,
    /// SIP UPDATE SipMethod.
    Update,
    /// SIP REFER SipMethod.
    Refer,
    /// SIP PRACK SipMethod.
    Prack,
    /// SIP MESSAGE SipMethod.
    Message,
    /// SIP PUBLISH SipMethod.
    Publish,
    /// An unknown SIP method.
    Unknown,
}

impl SipMethod {
    /// Returns the byte representation of a method.
    pub fn as_bytes(&self) -> &'static [u8] {
        self.as_str().as_bytes()
    }

    /// Returns the string representation of a method.
    #[inline(always)]
    pub fn as_str(&self) -> &'static str {
        match self {
            SipMethod::Invite => "INVITE",
            SipMethod::Ack => "ACK",
            SipMethod::Bye => "BYE",
            SipMethod::Cancel => "CANCEL",
            SipMethod::Register => "REGISTER",
            SipMethod::Options => "OPTIONS",
            SipMethod::Info => "INFO",
            SipMethod::Notify => "NOTIFY",
            SipMethod::Subscribe => "SUBSCRIBE",
            SipMethod::Update => "UPDATE",
            SipMethod::Refer => "REFER",
            SipMethod::Prack => "PRACK",
            SipMethod::Message => "MESSAGE",
            SipMethod::Publish => "PUBLISH",
            SipMethod::Unknown => "UNKNOWN-SipMethod",
        }
    }

    /// Returns `true` if this method can establish a dialog
    pub const fn can_establish_a_dialog(&self) -> bool {
        matches!(self, SipMethod::Invite)
    }
}

impl From<&[u8]> for SipMethod {
    fn from(value: &[u8]) -> Self {
        match value {
            b"INVITE" => SipMethod::Invite,
            b"CANCEL" => SipMethod::Cancel,
            b"ACK" => SipMethod::Ack,
            b"BYE" => SipMethod::Bye,
            b"REGISTER" => SipMethod::Register,
            b"OPTIONS" => SipMethod::Options,
            b"INFO" => SipMethod::Info,
            b"NOTIFY" => SipMethod::Notify,
            b"SUBSCRIBE" => SipMethod::Subscribe,
            b"UPDATE" => SipMethod::Update,
            b"REFER" => SipMethod::Refer,
            b"PRACK" => SipMethod::Prack,
            b"MESSAGE" => SipMethod::Message,
            b"PUBLISH" => SipMethod::Publish,
            _ => SipMethod::Unknown,
        }
    }
}

impl fmt::Display for SipMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
