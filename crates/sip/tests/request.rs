use sip::headers::{Header, SipHeaderParser, Via};

use sip::parser;

fn test() {
    let buff = b"SIP/2.0 200 OK\r\n";
     let parsed = Via::from_bytes(buff);
}