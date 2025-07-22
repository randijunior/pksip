use std::fmt;

const TP_UDP: &str = "UDP";
const TP_TCP: &str = "TCP";
const TP_TLS: &str = "TLS";
const TP_SCTP: &str = "SCTP";
const TP_WS: &str = "WS";
const TP_UNKNOWN: &str = "TP_UNKNOWN";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// An SIP Arc<dyn Transport> Type.
pub enum TransportProtocol {
    #[default]
    /// `UDP` transport.
    Udp,
    /// `TCP` transport.
    Tcp,
    /// `WebSocket` transport.
    Ws,
    /// `TLS` transport.
    Tls,
    /// `SCTP` transport.
    Sctp,
    /// UNKNOW transport.
    Unknown,
}

impl TransportProtocol {
    /// Returns the default port number associated with the transport protocol.
    ///
    /// - `UDP`, `TCP`, and `SCTP` use port `5060` by default.
    /// - `TLS` uses port `5061`.
    /// - `WS` uses port `80`.
    /// - `Unknown` returns `0` to indicate no default.
    #[inline]
    pub const fn get_port(&self) -> u16 {
        match self {
            TransportProtocol::Udp | TransportProtocol::Tcp | TransportProtocol::Sctp => 5060,
            TransportProtocol::Tls => 5061,
            TransportProtocol::Ws => 80,
            TransportProtocol::Unknown => 0,
        }
    }
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportProtocol::Udp => f.write_str(TP_UDP),
            TransportProtocol::Tcp => f.write_str(TP_TCP),
            TransportProtocol::Ws => f.write_str(TP_WS),
            TransportProtocol::Tls => f.write_str(TP_TLS),
            TransportProtocol::Sctp => f.write_str(TP_SCTP),
            TransportProtocol::Unknown => f.write_str(TP_UNKNOWN),
        }
    }
}

impl TransportProtocol {
    /// Returns the transport string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportProtocol::Udp => TP_UDP,
            TransportProtocol::Tcp => TP_TCP,
            TransportProtocol::Ws => TP_WS,
            TransportProtocol::Tls => TP_TLS,
            TransportProtocol::Sctp => TP_SCTP,
            TransportProtocol::Unknown => TP_UNKNOWN,
        }
    }
}

impl From<&str> for TransportProtocol {
    fn from(s: &str) -> Self {
        s.as_bytes().into()
    }
}

impl From<&[u8]> for TransportProtocol {
    fn from(b: &[u8]) -> Self {
        match b {
            b"UDP" | b"udp" => TransportProtocol::Udp,
            b"TCP" | b"tcp" => TransportProtocol::Tcp,
            b"WS" | b"ws" => TransportProtocol::Ws,
            b"TLS" | b"tls" => TransportProtocol::Tls,
            b"SCTP" | b"sctp" => TransportProtocol::Sctp,
            _ => TransportProtocol::Unknown,
        }
    }
}
