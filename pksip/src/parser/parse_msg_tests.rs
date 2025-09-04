use super::*;
use crate::filter_map_header;
use crate::find_map_header;
use crate::transport::TransportType;

#[test]
fn test_parse_request() {
    let buf = concat! {
        "INVITE sip:bob@biloxi.example.com SIP/2.0\r\n",
        "Via: SIP/2.0/TCP client.atlanta.example.com:5060;branch=z9hG4bK74b43\r\n",
        "Max-Forwards: 70\r\n",
        "Route: <sip:ss1.atlanta.example.com;lr>\r\n",
        "From: Alice <sip:alice@atlanta.example.com>;tag=9fxced76sl\r\n",
        "To: Bob <sip:bob@biloxi.example.com>\r\n",
        "Call-ID: 3848276298220188511@atlanta.example.com\r\n",
        "CSeq: 1 INVITE\r\n",
        "Contact: <sip:alice@client.atlanta.example.com;transport=tcp>\r\n",
        "Content-Type: application/sdp\r\n",
        "Content-Length: 151\r\n",
        "\r\n",
        "v=0\r\n",
        "o=alice 2890844526 2890844526 IN IP4 client.atlanta.example.com\r\n",
        "s=-\r\n",
        "c=IN IP4 192.0.2.101\r\n",
        "t=0 0\r\n",
        "m=audio 49172 RTP/AVP 0\r\n",
        "a=rtpmap:0 PCMU/8000\r\n"
    };

    let msg = Parser::parse_sip_msg(buf).unwrap();
    let req = msg.request().unwrap();

    assert_eq!(req.req_line.method, SipMethod::Invite);
    assert_eq!(req.req_line.uri.to_string(), "sip:bob@biloxi.example.com");

    let via = find_map_header!(req.headers, Via).unwrap();
    assert_eq!(via.transport(), TransportType::Tcp);
    assert_eq!(via.sent_by().to_string(), "client.atlanta.example.com:5060");
    assert_eq!(via.branch().unwrap(), "z9hG4bK74b43");

    let maxfowards = find_map_header!(req.headers, MaxForwards).unwrap();
    assert_eq!(maxfowards.max_fowards(), 70);

    let route = find_map_header!(req.headers, Route).unwrap();
    assert_eq!(route.addr.uri.to_string(), "sip:ss1.atlanta.example.com;lr");

    let from = find_map_header!(req.headers, From).unwrap();
    assert_eq!(from.display(), Some("Alice"));
    assert_eq!(from.tag().as_deref(), Some("9fxced76sl"));

    let to = find_map_header!(req.headers, To).unwrap();
    assert_eq!(to.display(), Some("Bob"));
    assert_eq!(to.uri().to_string(), "sip:bob@biloxi.example.com");

    let call_id = find_map_header!(req.headers, CallId).unwrap();
    assert_eq!(call_id.id(), "3848276298220188511@atlanta.example.com");

    let cseq = find_map_header!(req.headers, CSeq).unwrap();
    assert_eq!(cseq.cseq, 1);
    assert_eq!(cseq.method, SipMethod::Invite);

    let contact = find_map_header!(req.headers, Contact).unwrap();
    let host_str = contact.uri.uri().host_port.host_as_str();
    assert_eq!(host_str, "client.atlanta.example.com");

    let content_type = find_map_header!(req.headers, ContentType).unwrap();
    assert_eq!(content_type.media_type().to_string(), "application/sdp");

    let content_length = find_map_header!(req.headers, ContentLength).unwrap();
    assert_eq!(content_length.clen(), 151);

    assert_eq!(
        req.body.as_deref().unwrap(),
        concat!(
            "v=0\r\n",
            "o=alice 2890844526 2890844526 IN IP4 client.atlanta.example.com\r\n",
            "s=-\r\n",
            "c=IN IP4 192.0.2.101\r\n",
            "t=0 0\r\n",
            "m=audio 49172 RTP/AVP 0\r\n",
            "a=rtpmap:0 PCMU/8000\r\n"
        )
        .as_bytes()
    );
}

#[test]
fn test_parse_request_without_body() {
    let buf = concat! {
        "INVITE sip:bob@example.com SIP/2.0\r\n",
        "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
        "Max-Forwards: 70\r\n",
        "To: Bob <sip:bob@example.com>\r\n",
        "From: Alice <sip:alice@example.com>;tag=1928301774\r\n",
        "Call-ID: a84b4c76e66710\r\n",
        "CSeq: 314159 INVITE\r\n",
        "Contact: <sip:alice@example.com>\r\n",
        "Content-Length: 0\r\n",
        "\r\n"
    };

    let msg = Parser::parse_sip_msg(buf).unwrap();
    let req = msg.request().unwrap();

    assert_eq!(req.req_line.method, SipMethod::Invite);
    assert_eq!(req.req_line.uri.to_string(), "sip:bob@example.com");

    let via = find_map_header!(req.headers, Via).unwrap();
    assert_eq!(via.transport(), TransportType::Udp);
    assert_eq!(via.sent_by().to_string(), "pc33.atlanta.com");
    assert_eq!(via.branch().unwrap(), "z9hG4bK776asdhds");

    let maxfowards = find_map_header!(req.headers, MaxForwards).unwrap();
    assert_eq!(maxfowards.max_fowards(), 70);

    let to = find_map_header!(req.headers, To).unwrap();
    assert_eq!(to.uri().to_string(), "sip:bob@example.com");
    assert_eq!(to.display(), Some("Bob"));

    let from = find_map_header!(req.headers, From).unwrap();
    assert_eq!(from.display(), Some("Alice"));
    assert_eq!(from.uri().to_string(), "sip:alice@example.com");

    let call_id = find_map_header!(req.headers, CallId).unwrap();
    assert_eq!(call_id.id(), "a84b4c76e66710");

    let cseq = find_map_header!(req.headers, CSeq).unwrap();
    assert_eq!(cseq.cseq, 314159);

    let contact = find_map_header!(req.headers, Contact).unwrap();
    assert_eq!(contact.uri.to_string(), "<sip:alice@example.com>");

    let content_length = find_map_header!(req.headers, ContentLength).unwrap();
    assert_eq!(content_length.clen(), 0);
}

#[test]
fn test_parse_response() {
    let buf = concat! {
        "SIP/2.0 200 OK\r\n",
        "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
        "From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n",
        "To: Bob <sip:bob@example.com>;tag=a6c85cf\r\n",
        "Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n",
        "CSeq: 314159 INVITE\r\n",
        "Contact: <sip:bob@biloxi.com>\r\n",
        "Content-Type: application/sdp\r\n",
        "Content-Length: 131\r\n",
        "\r\n",
        "v=0\r\n",
        "o=bob 2808844564 2808844564 IN IP4 biloxi.com\r\n",
        "s=-\r\n",
        "c=IN IP4 biloxi.com\r\n",
        "t=0 0\r\n",
        "m=audio 7078 RTP/AVP 0\r\n",
        "a=rtpmap:0 PCMU/8000\r\n"
    };

    let msg = Parser::parse_sip_msg(buf).unwrap();
    let resp = msg.response().unwrap();

    assert_eq!(resp.code().as_u16(), 200);
    assert_eq!(resp.reason(), "OK");

    let via = find_map_header!(resp.headers, Via).unwrap();
    assert_eq!(via.transport(), TransportType::Udp);
    assert_eq!(via.sent_by().to_string(), "pc33.atlanta.com");

    let content_type = find_map_header!(resp.headers, ContentType).unwrap();
    assert_eq!(content_type.media_type().to_string(), "application/sdp");

    let content_length = find_map_header!(resp.headers, ContentLength).unwrap();
    assert_eq!(content_length.clen(), 131);

    assert_eq!(
        resp.body.as_deref().unwrap(),
        concat!(
            "v=0\r\n",
            "o=bob 2808844564 2808844564 IN IP4 biloxi.com\r\n",
            "s=-\r\n",
            "c=IN IP4 biloxi.com\r\n",
            "t=0 0\r\n",
            "m=audio 7078 RTP/AVP 0\r\n",
            "a=rtpmap:0 PCMU/8000\r\n"
        )
        .as_bytes()
    );
}

#[test]
fn test_parse_response_without_body() {
    let buf = concat! {
        "SIP/2.0 200 OK\r\n",
        "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
        "Max-Forwards: 70\r\n",
        "To: Bob <sip:bob@example.com>\r\n",
        "From: Alice <sip:alice@example.com>;tag=1928301774\r\n",
        "Call-ID: a84b4c76e66710\r\n",
        "CSeq: 314159 INVITE\r\n",
        "Content-Length: 0\r\n\r\n"
    };

    let msg = Parser::parse_sip_msg(buf).unwrap();
    let resp = msg.response().unwrap();

    assert_eq!(resp.code().as_u16(), 200);
    assert_eq!(resp.reason(), "OK");

    let via = find_map_header!(resp.headers, Via).unwrap();
    assert_eq!(via.transport(), TransportType::Udp);
    assert_eq!(via.sent_by().to_string(), "pc33.atlanta.com");

    let maxfowards = find_map_header!(resp.headers, MaxForwards).unwrap();
    assert_eq!(maxfowards.max_fowards(), 70);

    let to = find_map_header!(resp.headers, To).unwrap();
    assert_eq!(to.uri().to_string(), "sip:bob@example.com");
    assert_eq!(to.display(), Some("Bob"));

    let from = find_map_header!(resp.headers, From).unwrap();
    assert_eq!(from.display(), Some("Alice"));
    assert_eq!(from.uri().to_string(), "sip:alice@example.com");

    let call_id = find_map_header!(resp.headers, CallId).unwrap();
    assert_eq!(call_id.id(), "a84b4c76e66710");

    let cseq = find_map_header!(resp.headers, CSeq).unwrap();
    assert_eq!(cseq.cseq, 314159);
    assert_eq!(cseq.method, SipMethod::Invite);

    let content_length = find_map_header!(resp.headers, ContentLength).unwrap();
    assert_eq!(content_length.clen(), 0);
}

#[test]
fn test_parse_request_with_multiple_via_headers() {
    let buf = concat! {
        "REGISTER sip:registrar.example.com SIP/2.0\r\n",
        "Via: SIP/2.0/UDP host1.example.com;branch=z9hG4bK111\r\n",
        "Via: SIP/2.0/UDP host2.example.com;branch=z9hG4bK222\r\n",
        "Via: SIP/2.0/UDP host3.example.com;branch=z9hG4bK333\r\n",
        "Max-Forwards: 70\r\n",
        "To: <sip:alice@example.com>\r\n",
        "From: <sip:alice@example.com>;tag=1928301774\r\n",
        "Call-ID: manyvias@atlanta.com\r\n",
        "CSeq: 42 REGISTER\r\n",
        "Contact: <sip:alice@pc33.atlanta.com>\r\n",
        "Content-Length: 0\r\n"
    };

    let msg = Parser::parse_sip_msg(buf).unwrap();
    let req = msg.request().unwrap();

    assert_eq!(req.req_line.method, SipMethod::Register);
    assert_eq!(req.req_line.uri.to_string(), "sip:registrar.example.com");

    let vias: Vec<_> = filter_map_header!(req.headers, Via).collect();
    assert_eq!(vias.len(), 3);
    assert_eq!(vias[0].sent_by().to_string(), "host1.example.com");
    assert_eq!(vias[0].branch().unwrap(), "z9hG4bK111");
    assert_eq!(vias[1].sent_by().to_string(), "host2.example.com");
    assert_eq!(vias[1].branch().unwrap(), "z9hG4bK222");
    assert_eq!(vias[2].sent_by().to_string(), "host3.example.com");
    assert_eq!(vias[2].branch().unwrap(), "z9hG4bK333");

    let max_forwards = find_map_header!(req.headers, MaxForwards).unwrap();
    assert_eq!(max_forwards.max_fowards(), 70);

    let to = find_map_header!(req.headers, To).unwrap();
    assert_eq!(to.uri().to_string(), "sip:alice@example.com");

    let from = find_map_header!(req.headers, From).unwrap();
    assert_eq!(from.uri().to_string(), "sip:alice@example.com");
    assert_eq!(from.tag().as_deref(), Some("1928301774"));

    let call_id = find_map_header!(req.headers, CallId).unwrap();
    assert_eq!(call_id.id(), "manyvias@atlanta.com");

    let cseq = find_map_header!(req.headers, CSeq).unwrap();
    assert_eq!(cseq.cseq, 42);
    assert_eq!(cseq.method, SipMethod::Register);

    let contact = find_map_header!(req.headers, Contact).unwrap();
    assert_eq!(contact.uri.to_string(), "<sip:alice@pc33.atlanta.com>");

    let content_length = find_map_header!(req.headers, ContentLength).unwrap();
    assert_eq!(content_length.clen(), 0);

    assert!(req.body.is_none());
}

#[test]
fn test_header_with_multi_params() {
    let buf = concat! {
        "OPTIONS sip:bob@example.com SIP/2.0\r\n",
        "Via: SIP/2.0/UDP folded.example.com;branch=z9hG4bKfolded\r\n",
        "Max-Forwards: 70\r\n",
        "To: <sip:bob@example.com>\r\n",
        "From: <sip:alice@atlanta.com>;tag=1928301774\r\n",
        "Call-ID: foldedoptions@atlanta.com\r\n",
        "CSeq: 100 OPTIONS\r\n",
        "Contact: <sip:alice@atlanta.com>;",
        " param1=value1;",
        " param2=value2;",
        " param3=value3;",
        " param4=value4\r\n",
        "Content-Length: 0\r\n\r\n"
    };

    let msg = Parser::parse_sip_msg(buf).unwrap();
    let req = msg.request().unwrap();

    assert_eq!(req.req_line.method, SipMethod::Options);
    assert_eq!(req.req_line.uri.to_string(), "sip:bob@example.com");

    let via = find_map_header!(req.headers, Via).unwrap();
    assert_eq!(via.transport(), TransportType::Udp);
    assert_eq!(via.sent_by().to_string(), "folded.example.com");

    let maxfowards = find_map_header!(req.headers, MaxForwards).unwrap();
    assert_eq!(maxfowards.max_fowards(), 70);

    let to = find_map_header!(req.headers, To).unwrap();
    assert_eq!(to.uri().to_string(), "sip:bob@example.com");

    let from = find_map_header!(req.headers, From).unwrap();
    assert_eq!(from.uri().to_string(), "sip:alice@atlanta.com");

    let call_id = find_map_header!(req.headers, CallId).unwrap();
    assert_eq!(call_id.id(), "foldedoptions@atlanta.com");

    let cseq = find_map_header!(req.headers, CSeq).unwrap();
    assert_eq!(cseq.cseq, 100);
    assert_eq!(cseq.method, SipMethod::Options);

    let contact = find_map_header!(req.headers, Contact).unwrap();
    let params = contact.param.as_ref().unwrap();
    assert_eq!(contact.uri.to_string(), "<sip:alice@atlanta.com>");
    assert_eq!(params.get_named("param1"), Some("value1"));
    assert_eq!(params.get_named("param2"), Some("value2"));
    assert_eq!(params.get_named("param3"), Some("value3"));
    assert_eq!(params.get_named("param4"), Some("value4"));

    let content_length = find_map_header!(req.headers, ContentLength).unwrap();
    assert_eq!(content_length.clen(), 0);
}

#[test]
fn test_parse_request_with_invalid_uri() {
    let raw_msg = concat! {
        "INVITE bob@biloxi.com SIP/2.0\r\n",
        "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
        "Max-Forwards: 70\r\n",
        "To: Bob <sip:bob@biloxi.com>\r\n",
        "From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n",
        "Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n",
        "CSeq: 314159 INVITE\r\n",
        "Contact: <sip:alice@pc33.atlanta.com>\r\n",
        "Content-Type: application/sdp\r\n",
        "Content-Length: 4\r\n",
        "\r\n",
        "Test\r\n",
    };

    let mut parser = Parser::new(raw_msg.as_bytes());

    assert!(parser.parse().is_err());
}
