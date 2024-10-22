use sip_rs::headers::routing::via::Via;
use sip_rs::headers::SipHeaderParser;
use sip_rs::parser;

fn test() {
    let buff = b"SIP/2.0 200 OK\r\n";
    let parsed = Via::from_bytes(buff);
}
