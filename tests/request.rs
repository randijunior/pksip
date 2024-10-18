use sip_rs::parser;

fn test() {
    let buff = b"SIP/2.0 200 OK\r\n";
    let parsed = parser::parse(buff);
}