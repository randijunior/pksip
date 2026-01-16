use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pksip::parser::Parser;

fn bench_parse_sip_msg(c: &mut Criterion) {
    let buf = b"INVITE sip:bob@biloxi.example.com SIP/2.0\r\n
Via: SIP/2.0/TCP client.atlanta.example.com:5060;ttl=65;branch=z9hG4bK74bf9\r\n
Max-Forwards: 70\r\n
From: Alice <sip:alice@atlanta.example.com>;tag=9fxced76sl\r\n
To: Bob <sip:bob@biloxi.example.com>\r\n
Call-ID: 3848276298220188511@atlanta.example.com\r\n
CSeq: 2 INVITE\r\n
Contact: <sip:alice@client.atlanta.example.com;transport=tcp>\r\n
Diversion: Carol <sip:carol@atlanta.example.com>;privacy=off;reason=no-answer;counter=1;screen=no\r\n
Remote-Party-ID: Alice <sip:alice@atlanta.example.com>\r\n
P-Asserted-Identity: Alice <sip:alice@atlanta.example.com>\r\n
P-Charge-Info: <sip:eve@atlanta.example.com>\r\n
P-Source-Device: 216.3.128.12\r\n
Content-Type: application/sdp\r\n
Content-Length: 151\r\n
X-BroadWorks-DNC: network-address=sip:+9876543210@127.0.0.101;user=phone\r\n
User-Agent: X-Lite release 1104o stamp 56125\r\n\r\n";

    c.bench_function("parse invite with sdp", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(buf));
            let msg = parser.parse_sip_msg().unwrap();
            black_box(msg);
        });
    });
}

criterion_group!(benches, bench_parse_sip_msg);
criterion_main!(benches);
