use assert_matches::assert_matches;
use sip::{
    headers::Header,
    message::{Host, Scheme, SipMethod, TransportProtocol},
};

#[test]
fn test_parse_msg_1() {
    let parsed = sip::parser::parse_sip_msg(
        b"INVITE sip:bob@biloxi.com SIP/2.0\r\n\
    Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n\
    Max-Forwards: 70\r\n\
    To: Bob <sip:bob@biloxi.com>\r\n\
    From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n\
    Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n\
    CSeq: 314159 INVITE\r\n\
    Contact: <sip:alice@pc33.atlanta.com>\r\n\
    Content-Type: application/sdp\r\n\
    Content-Length: 142\r\n\
    \r\n\
    v=0\r\n\
    o=alice 2890844526 2890844526 IN IP4 pc33.atlanta.com\r\n\
    s=Session SDP\r\n\
    c=IN IP4 pc33.atlanta.com\r\n\
    t=0 0\r\n\
    m=audio 49170 RTP/AVP 0\r\n\
    a=rtpmap:0 PCMU/8000\r\n",
    );
    let parsed = parsed.unwrap();
    let req = parsed.request().unwrap();
    let mut iter = req.headers.iter();

    assert_eq!(req.req_line.method, SipMethod::Invite);
    assert_eq!(req.req_line.uri.user.as_ref().unwrap().get_user(), "bob");
    assert_eq!(req.req_line.uri.host_port.is_domain(), true);
    assert_eq!(req.req_line.uri.host_port.host_as_str(), "biloxi.com");

    assert_matches!(
        iter.next(),
        Some(Header::Via(via)) => {
            assert_eq!(via.transport, TransportProtocol::UDP);
            assert_eq!(via.sent_by.host, Host::DomainName("pc33.atlanta.com".into()));
            assert_eq!(via.sent_by.port, None);
            assert_eq!(via.branch, Some("z9hG4bK776asdhds".into()));
        }
    );
    assert_matches!(iter.next(), Some(Header::MaxForwards(m)) => {
        assert_eq!(m.max_fowards(), 70);
    });
    assert_matches!(iter.next(), Some(Header::To(to)) => {
        let name_addr = to.uri.name_addr().unwrap();
        assert_eq!(name_addr.display, Some("Bob".into()));
        assert_eq!(name_addr.uri.host_port.host, Host::DomainName("biloxi.com".into()));
        assert_eq!(name_addr.uri.scheme, Scheme::Sip);
    });
    assert_matches!(iter.next(), Some(Header::From(from)) => {
        let name_addr = from.uri.name_addr().unwrap();
        assert_eq!(name_addr.display, Some("Alice".into()));
        assert_eq!(name_addr.uri.host_port.host, Host::DomainName("atlanta.com".into()));
        assert_eq!(name_addr.uri.scheme, Scheme::Sip);
        assert_eq!(from.tag, Some("1928301774".into()));
    });
    assert_matches!(iter.next(), Some(Header::CallId(c)) => {
        assert_eq!(c.id(), "a84b4c76e66710@pc33.atlanta.com");
    });
    assert_matches!(iter.next(), Some(Header::CSeq(c)) => {
        assert_eq!(c.cseq, 314159);
        assert_eq!(c.method, SipMethod::Invite);
    });
    assert_matches!(iter.next(), Some(Header::Contact(c)) => {
        let uri = &c.uri().unwrap().name_addr().unwrap().uri;
        assert_eq!(uri
            .host_port
            .host, Host::DomainName("pc33.atlanta.com".into()));
        assert_eq!(uri.user.as_ref().unwrap().get_user(), "alice");
    });
    assert_matches!(iter.next(), Some(Header::ContentType(c)) => {
        assert_eq!(c.0.mimetype.mtype, "application".into());
        assert_eq!(c.0.mimetype.subtype, "sdp".into());
    });
    assert_matches!(iter.next(), Some(Header::ContentLength(c)) => {
        assert_eq!(c.0, 142);
    });
    assert_matches!(iter.next(), None);

    assert_eq!(
        req.body,
        Some(
            "v=0\r\n\
    o=alice 2890844526 2890844526 IN IP4 pc33.atlanta.com\r\n\
    s=Session SDP\r\n\
    c=IN IP4 pc33.atlanta.com\r\n\
    t=0 0\r\n\
    m=audio 49170 RTP/AVP 0\r\n\
    a=rtpmap:0 PCMU/8000\r\n"
                .as_bytes()
                .into()
        )
    );
}

#[test]
fn test_parse_msg_2() {
    let parsed = sip::parser::parse_sip_msg(
        b"REGISTER sip:registrar.biloxi.com SIP/2.0\r\n\
        Via: SIP/2.0/UDP bobspc.biloxi.com:5060;branch=z9hG4bKnashds7\r\n\
        Max-Forwards: 70\r\n\
        To: Bob <sip:bob@biloxi.com>\r\n\
        From: Bob <sip:bob@biloxi.com>;tag=456248\r\n\
        Call-ID: 843817637684230@998sdasdh09\r\n\
        CSeq: 1826 REGISTER\r\n\
        Contact: <sip:bob@192.0.2.4>\r\n\
        Expires: 7200\r\n\
        Content-Length: 0\r\n",
    );

    let parsed = parsed.unwrap();
    let req = parsed.request().unwrap();
    let mut iter = req.headers.iter();

    assert_eq!(req.req_line.method, SipMethod::Register);
    assert_eq!(req.req_line.uri.host_port.is_domain(), true);
    assert_eq!(
        req.req_line.uri.host_port.host_as_str(),
        "registrar.biloxi.com"
    );

    assert_matches!(
        iter.next(),
        Some(Header::Via(via)) => {
            assert_eq!(via.transport, TransportProtocol::UDP);
            assert_eq!(via.sent_by.host, Host::DomainName("bobspc.biloxi.com".into()));
            assert_eq!(via.sent_by.port, Some(5060));
            assert_eq!(via.branch, Some("z9hG4bKnashds7".into()));
        }
    );
    assert_matches!(iter.next(), Some(Header::MaxForwards(m)) => {
        assert_eq!(m.max_fowards(), 70);
    });

    assert_matches!(iter.next(), Some(Header::To(to)) => {
        let name_addr = to.uri.name_addr().unwrap();
        assert_eq!(name_addr.display, Some("Bob".into()));
        assert_eq!(name_addr.uri.host_port.host, Host::DomainName("biloxi.com".into()));
        assert_eq!(name_addr.uri.scheme, Scheme::Sip);
    });

    assert_matches!(iter.next(), Some(Header::From(from)) => {
        let name_addr = from.uri.name_addr().unwrap();
        assert_eq!(name_addr.display, Some("Bob".into()));
        assert_eq!(name_addr.uri.host_port.host, Host::DomainName("biloxi.com".into()));
        assert_eq!(name_addr.uri.scheme, Scheme::Sip);
        assert_eq!(from.tag, Some("456248".into()));
    });

    assert_matches!(iter.next(), Some(Header::CallId(c)) => {
        assert_eq!(c.id(), "843817637684230@998sdasdh09");
    });

    assert_matches!(iter.next(), Some(Header::CSeq(c)) => {
        assert_eq!(c.cseq, 1826);
        assert_eq!(c.method, SipMethod::Register);
    });
    assert_matches!(iter.next(), Some(Header::Contact(c)) => {
        let uri = &c.uri().unwrap().name_addr().unwrap().uri;
        assert_eq!(uri
            .host_port
            .host, Host::IpAddr("192.0.2.4".parse().unwrap()));
        assert_eq!(uri.user.as_ref().unwrap().get_user(), "bob");
    });
    assert_matches!(iter.next(), Some(Header::Expires(e)) => {
        assert_eq!(e.0, 7200);
    });
    assert_matches!(iter.next(), Some(Header::ContentLength(c)) => {
        assert_eq!(c.0, 0);
    });
    assert_matches!(iter.next(), None);
}
