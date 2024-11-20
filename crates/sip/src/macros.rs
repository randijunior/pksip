macro_rules! b_map {
    ($name:ident => $( $slice:expr ),+) => {
        const $name: [bool; 256] = {
            let mut arr = [false; 256];
            $(
                let slice = $slice;
                let mut i = 0;
                while i < slice.len() {
                    arr[slice[i] as usize] = true;
                    i += 1;
                }
            )*
            arr
        };
    };
}

macro_rules! parse_header_param {
    ($scanner:ident) => (
        $crate::macros::parse_param!(
            $scanner,
            $crate::headers::parse_header_param,
        )
    );

    ($scanner:ident, $($name:ident = $var:expr),*) => (
        $crate::macros::parse_param!(
            $scanner,
            $crate::headers::parse_header_param,
            $($name = $var),*
        )
    );
}

macro_rules! parse_param {
    (
        $scanner:ident,
        $func:expr,
        $($name:ident = $var:expr),*
    ) =>  {{
        scanner::space!($scanner);
        match $scanner.peek() {
            Some(&b';') => {
                let mut params = $crate::uri::Params::new();
                while let Some(&b';') = $scanner.peek() {
                        // take ';' character
                        $scanner.next();
                        let param = $func($scanner)?;
                        $(
                            if param.0 == $name {
                                $var = param.1;
                                scanner::space!($scanner);
                                continue;
                            }
                        )*
                        params.set(param.0, param.1.unwrap_or(""));
                        scanner::space!($scanner);
                    }
                    if params.is_empty() {
                        None
                    } else {
                        Some(params)
                    }
                },
                _ => {
                    None
                }
            }
        }};
    }

macro_rules! parse_header_list {
    ($scanner:ident => $body:expr) => {{
        let mut hdr_itens = Vec::new();
        $crate::macros::parse_comma_separated!($scanner => {
            hdr_itens.push($body);
        });
        hdr_itens
    }};
}

macro_rules! parse_comma_separated {
    ($scanner:ident => $body:expr) => {{
        scanner::space!($scanner);
        $body

        while let Some(b',') = $scanner.peek() {
            $scanner.next();
            scanner::space!($scanner);
            $body
        }
    }};
}

macro_rules! sip_parse_error {
    ($message:expr) => {{
        Err(crate::parser::error::SipParserError::from($message))
    }};
}

pub(crate) use b_map;
pub(crate) use parse_comma_separated;
pub(crate) use parse_header_list;
pub(crate) use parse_header_param;
pub(crate) use parse_param;

pub(crate) use sip_parse_error;

