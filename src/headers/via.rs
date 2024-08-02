/*
Via               =  ( "Via" / "v" ) HCOLON via-parm *(COMMA via-parm)
via-parm          =  sent-protocol LWS sent-by *( SEMI via-params )
via-params        =  via-ttl / via-maddr
                     / via-received / via-branch
                     / via-extension
via-ttl           =  "ttl" EQUAL ttl
via-maddr         =  "maddr" EQUAL host
via-received      =  "received" EQUAL (IPv4address / IPv6address)
via-branch        =  "branch" EQUAL token
via-extension     =  generic-param
sent-protocol     =  protocol-name SLASH protocol-version
                     SLASH transport
protocol-name     =  "SIP" / token
protocol-version  =  token
transport         =  "UDP" / "TCP" / "TLS" / "SCTP"
                     / other-transport
sent-by           =  host [ COLON port ]
ttl               =  1*3DIGIT ; 0 to 255
*/

use crate::{
    msg::Transport, uri::{GenericParams, HostPort}
};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ViaParams<'a> {
    ttl: Option<&'a str>,
    maddr: Option<&'a str>,
    received: Option<&'a str>,
    branch: Option<&'a str>,
    extension: Option<&'a str>,
    rport: Option<u16>,
}

impl<'a> ViaParams<'a> {
    pub fn set_branch(&mut self, branch: &'a str) {
        self.branch = Some(branch);
    }

    pub fn set_ttl(&mut self, ttl: &'a str) {
        self.ttl = Some(ttl);
    }
    pub fn set_maddr(&mut self, maddr: &'a str) {
        self.maddr = Some(maddr);
    }
    pub fn set_received(&mut self, received: &'a str) {
        self.received = Some(received);
    }
    pub fn set_rport(&mut self, rport: u16) {
        self.rport = Some(rport);
    }
}

// SIP Via Header
pub struct Via<'a> {
    transport: Transport,
    sent_by: HostPort<'a>,
    params: Option<ViaParams<'a>>,
    other_params: Option<GenericParams<'a>>,
}