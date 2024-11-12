use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sip::parser::SipParser;

const INVITE: &[u8] = b"INVITE sip:bob@biloxi.com SIP/2.0\r\n
Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKkjshdyff\r\n
To: Bob <sip:bob@biloxi.com>\r\n
From: Alice <sip:alice@atlanta.com>;tag=88sja8x\r\n
Max-Forwards: 70\r\n
Call-ID: 987asjd97y7atg\r\n
CSeq: 986759 INVITE\r\n";

fn request(c: &mut Criterion) {
    c.bench_function("request", |b| {
        b.iter(|| {
            black_box(|| {
                let _ = SipParser::parse(INVITE).unwrap();
            })
        });
    });
}

criterion_group!(
    benches,
    request,
);
criterion_main!(benches);