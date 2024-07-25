
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

use std::{collections::HashSet, net::IpAddr};

use crate::{msg::Transport, uri::{GenericParams, Host}};


pub struct Via<'a> {
    transport: Transport,
    sent_by: Host<'a>,
    ttl: u8,
    ttl_param: Option<&'a str>,
    maddr_param: Option<&'a str>,
    received_param: Option<&'a str>,
    branch_param: Option<&'a str>,
    extension_param: Option<&'a str>,
    other_params: Option<GenericParams<'a>>,
}