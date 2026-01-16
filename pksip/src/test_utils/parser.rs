#[macro_export]
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
