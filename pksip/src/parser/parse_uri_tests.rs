use super::*;

macro_rules! uri_test_ok {
    (name: $name:ident, input: $input:literal, expected: $expected:expr) => {
        #[test]
        fn $name() -> Result<()> {
            let uri = Parser::new($input).parse_sip_uri(true)?;

            assert_eq!($expected.scheme, uri.scheme());
            assert_eq!($expected.host_port.host, uri.host_port().host);
            assert_eq!($expected.host_port.port, uri.host_port().port);
            assert_eq!($expected.user, uri.user().cloned());

            assert_eq!($expected.transport_param, uri.transport_param());
            assert_eq!(&$expected.ttl_param, uri.ttl_param());
            assert_eq!(&$expected.method_param, uri.method_param());
            assert_eq!(&$expected.user_param, uri.user_param());
            assert_eq!($expected.lr_param, uri.lr_param());
            assert_eq!(&$expected.maddr_param, uri.maddr_param());

            if let Some(params) = uri.other_params() {
                assert!($expected.parameters.is_some(), "missing parameters!");
                for param in $expected.parameters.unwrap().iter() {
                    assert_eq!(params.get_named(param.name()), param.value());
                }
            }
            if let Some(headers) = uri.headers() {
                assert!($expected.headers.is_some(), "missing headers!");
                for param in $expected.headers.unwrap().iter() {
                    assert_eq!(headers.get_named(param.name()), param.value());
                }
            }

            Ok(())
        }
    };
}

uri_test_ok! {
    name: uri_test_1,
    input: "sip:biloxi.com",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .host("biloxi.com".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_2,
    input: "sip:biloxi.com:5060",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .host("biloxi.com:5060".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_3,
    input: "sip:a@b:5060",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("a", None))
        .host("b:5060".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_4,
    input: "sip:bob@biloxi.com:5060",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_5,
    input: "sip:bob@192.0.2.201:5060",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("192.0.2.201:5060".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_6,
    input: "sip:bob@[::1]:5060",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("[::1]:5060".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_7,
    input: "sip:bob:secret@biloxi.com",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", Some("secret")))
        .host("biloxi.com".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_8,
    input: "sip:bob:pass@192.0.2.201",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", Some("pass")))
        .host("192.0.2.201".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_9,
    input: "sip:bob@biloxi.com;foo=bar",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com".parse().unwrap())
        .param("foo", Some("bar"))
        .build()
}

uri_test_ok! {
    name: uri_test_10,
    input: "sip:bob@biloxi.com:5060;foo=bar",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .param("foo", Some("bar"))
        .build()
}

uri_test_ok! {
    name: uri_test_11,
    input: "sips:bob@biloxi.com:5060",
    expected: Uri::builder()
        .scheme(Scheme::Sips)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: uri_test_12,
    input: "sips:bob:pass@biloxi.com:5060",
    expected: Uri::builder()
        .scheme(Scheme::Sips)
        .user(UserInfo::new("bob", Some("pass")))
        .host("biloxi.com:5060".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: test_uri_11,
    input: "sip:bob@biloxi.com:5060;foo",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .param("foo", None)
        .host("biloxi.com:5060".parse().unwrap())
        .build()
}

uri_test_ok! {
    name: test_uri_12,
    input: "sip:bob@biloxi.com:5060;foo;baz=bar",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .param("baz", Some("bar"))
        .build()
}

uri_test_ok! {
    name: test_uri_13,
    input: "sip:bob@biloxi.com:5060;baz=bar;foo",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .param("baz", Some("bar"))
        .build()
}

uri_test_ok! {
    name: test_uri_14,
    input: "sip:bob@biloxi.com:5060;baz=bar;foo;a=b",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .param("baz", Some("bar"))
        .param("foo", None)
        .param("a", Some("b"))
        .build()
}

uri_test_ok! {
    name: test_uri_15,
    input: "sip:bob@biloxi.com?foo=bar",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com".parse().unwrap())
        .header("foo", Some("bar"))
        .build()
}

uri_test_ok! {
    name: test_uri_16,
    input: "sip:bob@biloxi.com?foo",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com".parse().unwrap())
        .header("foo", None)
        .build()
}

uri_test_ok! {
    name: test_uri_17,
    input: "sip:bob@biloxi.com:5060?foo=bar",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .header("foo", Some("bar"))
        .build()
}

uri_test_ok! {
    name: test_uri_18,
    input: "sip:bob@biloxi.com:5060?baz=bar&foo=&a=b",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .header("baz", Some("bar"))
        .header("foo", Some(""))
        .header("a", Some("b"))
        .build()
}

uri_test_ok! {
    name: test_uri_19,
    input: "sip:bob@biloxi.com:5060?foo=bar&baz",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com:5060".parse().unwrap())
        .header("foo", Some("bar"))
        .header("baz", None)
        .build()
}

uri_test_ok! {
    name: test_uri_20,
    input: "sip:bob@biloxi.com;foo?foo=bar",
    expected: Uri::builder()
        .scheme(Scheme::Sip)
        .user(UserInfo::new("bob", None))
        .host("biloxi.com".parse().unwrap())
        .param("foo", None)
        .header("foo", Some("bar"))
        .build()
}
