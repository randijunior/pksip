use std::fmt;

const TP_UDP: &str = "UDP";
const TP_TCP: &str = "TCP";
const TP_TLS: &str = "TLS";
const TP_SCTP: &str = "SCTP";
const TP_WS: &str = "WS";
const TP_UNKNOWN: &str = "TP_UNKNOWN";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// An SIP Transport Type.
pub enum TransportKind {
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

impl TransportKind {
    /// Returns the default port number associated with the transport protocol.
    ///
    /// - `UDP`, `TCP`, and `SCTP` use port `5060` by default.
    /// - `TLS` uses port `5061`.
    /// - `WS` uses port `80`.
    /// - `Unknown` returns `0` to indicate no default.
    #[inline]
    pub const fn get_port(&self) -> u16 {
        match self {
            TransportKind::Udp | TransportKind::Tcp | TransportKind::Sctp => 5060,
            TransportKind::Tls => 5061,
            TransportKind::Ws => 80,
            TransportKind::Unknown => 0,
        }
    }
}

impl fmt::Display for TransportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportKind::Udp => f.write_str(TP_UDP),
            TransportKind::Tcp => f.write_str(TP_TCP),
            TransportKind::Ws => f.write_str(TP_WS),
            TransportKind::Tls => f.write_str(TP_TLS),
            TransportKind::Sctp => f.write_str(TP_SCTP),
            TransportKind::Unknown => f.write_str(TP_UNKNOWN),
        }
    }
}

impl TransportKind {
    /// Returns the transport string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportKind::Udp => TP_UDP,
            TransportKind::Tcp => TP_TCP,
            TransportKind::Ws => TP_WS,
            TransportKind::Tls => TP_TLS,
            TransportKind::Sctp => TP_SCTP,
            TransportKind::Unknown => TP_UNKNOWN,
        }
    }
}

impl From<&str> for TransportKind {
    fn from(s: &str) -> Self {
        s.as_bytes().into()
    }
}

impl From<&[u8]> for TransportKind {
    fn from(b: &[u8]) -> Self {
        match b {
            b"UDP" | b"udp" => TransportKind::Udp,
            b"TCP" | b"tcp" => TransportKind::Tcp,
            b"WS" | b"ws" => TransportKind::Ws,
            b"TLS" | b"tls" => TransportKind::Tls,
            b"SCTP" | b"sctp" => TransportKind::Sctp,
            _ => TransportKind::Unknown,
        }
    }
}
