use std::{
    borrow::Cow,
    fmt,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::Arc,
};

use itertools::Itertools;

use crate::{
    error::{Error, Result},
    parser::ParseCtx,
};

use super::{Method, Params, TransportKind};

#[derive(Debug, PartialEq, Eq, Clone)]
/// A SIP URI.
///
/// Represents a Uniform Resource Identifier(URI) used in SIP messages, which can either be a plain `Uri`
/// or a `NameAddr` (a named address with optional display name).
///
/// # Examples
/// ```
/// use pksip::message::{Uri, NameAddr, SipUri};
///
/// let uri = Uri::from_static("sip:alice@example.com").unwrap();
/// let sip_uri = SipUri::Uri(uri);
///
/// let name_addr = NameAddr::from_static("\"Alice\" <sip:alice@example.com>").unwrap();
/// let named = SipUri::NameAddr(name_addr);
/// ```
pub enum SipUri<'a> {
    /// A plain SIP URI (e.g. `sip:user@example.com`)
    Uri(Uri<'a>),
    /// A named address (e.g. `"Alice" <sip:user@example.com>`)
    NameAddr(NameAddr<'a>),
}

impl std::fmt::Display for SipUri<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SipUri::Uri(uri) => write!(f, "{}", uri),
            SipUri::NameAddr(name_addr) => write!(f, "{}", name_addr),
        }
    }
}

impl<'a> SipUri<'a> {
    /// Returns a reference to the [`Uri`] if this is a [`SipUri::Uri`] variant.
    pub fn uri(&self) -> Option<&Uri> {
        if let SipUri::Uri(uri) = self {
            Some(uri)
        } else {
            None
        }
    }

    /// Returns a reference to the [`NameAddr`] if this is a [`SipUri::NameAddr`] variant.
    pub fn name_addr(&self) -> Option<&NameAddr> {
        if let SipUri::NameAddr(addr) = self {
            Some(addr)
        } else {
            None
        }
    }

    /// Returns the scheme of the uri.
    pub fn scheme(&self) -> Scheme {
        match self {
            SipUri::Uri(uri) => uri.scheme,
            SipUri::NameAddr(name_addr) => name_addr.uri.scheme,
        }
    }

    /// Returns the user part of the uri.
    pub fn user(&self) -> Option<&UriUser> {
        match self {
            SipUri::Uri(uri) => uri.user.as_ref(),
            SipUri::NameAddr(name_addr) => name_addr.uri.user.as_ref(),
        }
    }

    /// Returns a reference to the [`HostPort`] of the uri.
    pub fn host_port(&self) -> &HostPort {
        match self {
            SipUri::Uri(uri) => &uri.host_port,
            SipUri::NameAddr(name_addr) => &name_addr.uri.host_port,
        }
    }

    /// Returns the `transport` parameter.
    pub fn transport_param(&self) -> Option<TransportKind> {
        match self {
            SipUri::Uri(uri) => uri.transport_param,
            SipUri::NameAddr(name_addr) => name_addr.uri.transport_param,
        }
    }
    /// Returns the user parameter of the uri.
    pub fn user_param(&self) -> &Option<&'a str> {
        match self {
            SipUri::Uri(uri) => &uri.user_param,
            SipUri::NameAddr(name_addr) => &name_addr.uri.user_param,
        }
    }

    /// Returns the method parameter of the uri.
    pub fn method_param(&self) -> &Option<Method> {
        match self {
            SipUri::Uri(uri) => &uri.method_param,
            SipUri::NameAddr(name_addr) => &name_addr.uri.method_param,
        }
    }

    /// Returns the ttl parameter of the uri.
    pub fn ttl_param(&self) -> &Option<u8> {
        match self {
            SipUri::Uri(uri) => &uri.ttl_param,
            SipUri::NameAddr(name_addr) => &name_addr.uri.ttl_param,
        }
    }

    /// Returns the lr parameter of the uri.
    pub fn lr_param(&self) -> bool {
        match self {
            SipUri::Uri(uri) => uri.lr_param,
            SipUri::NameAddr(name_addr) => name_addr.uri.lr_param,
        }
    }

    /// Returns the maddr parameter of the uri.
    pub fn maddr_param(&self) -> &Option<&'a str> {
        match self {
            SipUri::Uri(uri) => &uri.maddr_param,
            SipUri::NameAddr(name_addr) => &name_addr.uri.maddr_param,
        }
    }

    /// Returns the other parameters of the uri.
    pub fn params(&self) -> Option<&Params<'a>> {
        match self {
            SipUri::Uri(uri) => uri.params.as_ref(),
            SipUri::NameAddr(name_addr) => name_addr.uri.params.as_ref(),
        }
    }

    /// Returns the header parameters of the uri.
    pub fn header_params(&self) -> Option<&Params<'a>> {
        match self {
            SipUri::Uri(uri) => uri.hdr_params.as_ref(),
            SipUri::NameAddr(name_addr) => name_addr.uri.hdr_params.as_ref(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Copy)]
/// A SIP URI scheme, either `sip` or `sips`.
///
/// Represents the scheme that appears in a SIP URI.
pub enum Scheme {
    #[default]
    /// An Sip uri scheme.
    Sip,
    /// An Sips uri scheme.
    Sips,
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
/// An SIP uri.
pub struct Uri<'a> {
    /// The uri scheme.
    pub scheme: Scheme,

    /// Optionaluser part of uri.
    pub user: Option<UriUser<'a>>,

    /// The uri host.
    pub host_port: HostPort,

    /// Optional user param.
    pub user_param: Option<&'a str>,

    /// Optional method param.
    pub method_param: Option<Method>,

    /// Optional transport param.
    pub transport_param: Option<TransportKind>,

    /// Optional ttl param.
    pub ttl_param: Option<u8>,

    /// Optional ttl param.
    pub lr_param: bool,

    /// Optional maddr param.
    pub maddr_param: Option<&'a str>,

    /// Other parameters.
    pub params: Option<Params<'a>>,

    /// Optional header parameters
    pub hdr_params: Option<Params<'a>>,
}

impl std::fmt::Display for Uri<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.scheme {
            Scheme::Sip => write!(f, "sip")?,
            Scheme::Sips => write!(f, "sips")?,
        }
        write!(f, ":")?;

        if let Some(user) = &self.user {
            write!(f, "{}", user.user)?;
            if let Some(pass) = user.pass {
                write!(f, ":{}", pass)?;
            }
            write!(f, "@")?;
        }
        write!(f, "{}", self.host_port)?;

        if let Some(user) = &self.user_param {
            write!(f, ";user={}", user)?;
        }
        if let Some(method) = &self.method_param {
            write!(f, ";method={}", method)?;
        }
        if let Some(maddr) = &self.maddr_param {
            write!(f, ";maddr={}", maddr)?;
        }
        if let Some(transport) = &self.transport_param {
            write!(f, ";transport={}", transport)?;
        }
        if let Some(ttl) = self.ttl_param {
            write!(f, ";ttl={}", ttl)?;
        }
        if self.lr_param {
            write!(f, ";lr")?;
        }
        if let Some(params) = &self.params {
            write!(f, ";{}", params)?;
        }
        if let Some(hdr_params) = &self.hdr_params {
            let formater = Itertools::format_with(hdr_params.iter(), "&", |it, f| {
                f(&format_args!("{}={}", it.name, it.value.unwrap_or("")))
            });
            write!(f, "?{}", formater)?;
        }

        Ok(())
    }
}

impl<'a> Uri<'a> {
    /// Creates an `Uri` instance witthout parameters.
    pub fn without_params(scheme: Scheme, user: Option<UriUser<'a>>, host_port: HostPort) -> Self {
        Uri {
            scheme,
            user,
            host_port,
            ..Default::default()
        }
    }

    /// Create a `Uri` with a static string.
    ///
    /// # Panic
    ///
    /// Panics if the string is not a legal sip URI.
    pub fn from_static(s: &'static str) -> Result<Self> {
        let mut p = ParseCtx::new(s.as_bytes());

        p.parse_uri(true)
    }
}

#[derive(Default)]
/// Builder for creating a new SIP URI.
pub struct UriBuilder<'a> {
    uri: Uri<'a>,
}

impl<'a> UriBuilder<'a> {
    /// Returns a builder to create an `UriBuilder`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the uri scheme.
    pub fn scheme(mut self, scheme: Scheme) -> Self {
        self.uri.scheme = scheme;
        self
    }

    /// Sets the user part of the uri.
    pub fn user(mut self, user: UriUser<'a>) -> Self {
        self.uri.user = Some(user);
        self
    }

    /// Sets the host of the uri.
    pub fn host(mut self, host_port: HostPort) -> Self {
        self.uri.host_port = host_port;
        self
    }

    /// Sets the user parameter of the uri.
    pub fn user_param(mut self, param: &'a str) -> Self {
        self.uri.user_param = Some(param);
        self
    }

    /// Sets the method parameter of the uri.
    pub fn method_param(mut self, param: Method) -> Self {
        self.uri.method_param = Some(param);
        self
    }

    /// Sets the transport parameter of the uri.
    pub fn transport_param(mut self, param: TransportKind) -> Self {
        self.uri.transport_param = Some(param);
        self
    }

    /// Sets the ttl parameter of the uri.
    pub fn ttl_param(mut self, param: &str) -> Self {
        self.uri.ttl_param = Some(param.parse().unwrap());
        self
    }

    /// Sets the lr parameter of the uri.
    pub fn lr_param(mut self, param: bool) -> Self {
        self.uri.lr_param = param;
        self
    }

    /// Sets the maddr parameter of the uri.
    pub fn maddr_param(mut self, param: &'a str) -> Self {
        self.uri.maddr_param = Some(param);
        self
    }

    /// Sets other parameters of the uri.
    pub fn params(mut self, params: Params<'a>) -> Self {
        self.uri.params = Some(params);
        self
    }

    /// Set generic parameter of the uri.
    pub fn param(mut self, name: &'a str, value: &'a str) -> Self {
        if let Some(params) = &mut self.uri.params {
            params.push(super::Param {
                name,
                value: value.into(),
            });
        } else {
            let mut params = Params::new();
            params.push(super::Param {
                name,
                value: value.into(),
            });
            self.uri.params = Some(params);
        }
        self
    }

    /// Set header parameter of the uri.
    pub fn header_param(mut self, name: &'a str, value: &'a str) -> Self {
        if let Some(hdr_params) = &mut self.uri.hdr_params {
            hdr_params.push(super::Param {
                name,
                value: value.into(),
            });
        } else {
            let mut hdr_params = Params::new();
            hdr_params.push(super::Param {
                name,
                value: value.into(),
            });
            self.uri.hdr_params = Some(hdr_params);
        }
        self
    }

    /// Finalize the builder into a `Uri`.
    pub fn get(self) -> Uri<'a> {
        self.uri
    }
}

/// Represents an SIP `name-addr`.
///  
/// Typically appear in `From`, `To`, and `Contact` header.
/// Contains an sip uri and a optional display part.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NameAddr<'a> {
    /// The optional display part.
    pub display: Option<&'a str>,
    /// The uri of the `name-addr`.
    pub uri: Uri<'a>,
}

impl<'a> NameAddr<'a> {
    /// Create a `NameAddr` from a static string.
    ///
    /// # Panic
    ///
    /// Panics if the string is not a legal sip message.
    pub fn from_static(s: &'static str) -> Result<Self> {
        let mut p = ParseCtx::new(s.as_bytes());

        p.parse_name_addr()
    }
}

impl std::fmt::Display for NameAddr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(display) = &self.display {
            write!(f, "{} ", display)?;
        }
        write!(f, "<{}>", self.uri)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Represents the user information component of a URI.
pub struct UriUser<'a> {
    /// The username part of the URI.
    pub user: &'a str,

    /// The optional password associated with the user.
    pub pass: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
/// Represents the host part of a URI, which can be either a domain name or an IP address.
pub enum Host {
    /// A domain name, such as `example.com`.
    DomainName(Arc<str>),

    /// An IP address, either IPv4 or IPv6.
    IpAddr(IpAddr),
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::DomainName(domain) => write!(f, "{domain}"),
            Host::IpAddr(ip_addr) => write!(f, "{ip_addr}"),
        }
    }
}

impl Host {
    /// Returns `true` if the host is an IP address (IPv4 or IPv6).
    pub fn is_ip_addr(&self) -> bool {
        match self {
            Host::DomainName(_) => false,
            Host::IpAddr(_) => true,
        }
    }

    /// Returns the string representation of the host as a `Cow<str>`.
    ///
    /// If the host is a domain name, this returns a borrowed string.
    /// If the host is an IP address, this returns an owned string created via formatting.
    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Host::DomainName(host) => Cow::Borrowed(host),
            Host::IpAddr(host) => Cow::Owned(host.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
/// Represents a combination of a host (domain or IP address) and an optional port.
pub struct HostPort {
    /// The host part, which may be a domain name or an IP address.
    pub host: Host,

    /// The optional port number.
    pub port: Option<u16>,
}

impl FromStr for HostPort {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut p = ParseCtx::new(s.as_bytes());

        p.parse_host_port()
    }
}

impl HostPort {
    /// Returns the IP address if the host is an IP address, otherwise `None`.
    pub fn ip_addr(&self) -> Option<IpAddr> {
        match self.host {
            Host::DomainName(_) => None,
            Host::IpAddr(ip_addr) => Some(ip_addr),
        }
    }

    /// Returns `true` if the host is an IP address.
    pub fn is_ip_addr(&self) -> bool {
        self.ip_addr().is_some()
    }
}

impl fmt::Display for HostPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.host {
            Host::DomainName(domain) => f.write_str(domain)?,
            Host::IpAddr(ip_addr) => write!(f, "{}", ip_addr)?,
        }
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        Ok(())
    }
}

impl From<Host> for HostPort {
    fn from(host: Host) -> Self {
        Self { host, port: None }
    }
}

impl HostPort {
    /// Creates a new `HostPort` from a host and optional port.
    pub fn new(host: Host, port: Option<u16>) -> Self {
        Self { host, port }
    }

    /// Returns `true` if the host is a domain name.
    pub fn is_domain(&self) -> bool {
        matches!(self.host, Host::DomainName(_))
    }

    /// Returns the string representation of the host.
    pub fn host_as_str(&self) -> Cow<'_, str> {
        self.host.as_str()
    }
}

impl Default for HostPort {
    fn default() -> Self {
        Self {
            host: Host::IpAddr(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            port: Some(5060),
        }
    }
}
