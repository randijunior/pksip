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
            $crate::parser::ParseCtx::parse_param,
        )
    );

    ($scanner:ident, $($name:ident = $var:expr),*) => (
        $crate::macros::parse_param!(
            $scanner,
            $crate::parser::ParseCtx::parse_param,
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
        $scanner.take_ws();
        match $scanner.peek() {
            Some(b';') => {
                let mut params = $crate::message::Params::new();
                while let Some(b';') = $scanner.peek() {
                        // take ';' character
                        $scanner.advance();
                        let param = $func($scanner)?;
                        $(
                            if param.name == $name {
                                $var = param.value;
                                $scanner.take_ws();
                                continue;
                            }
                        )*
                        params.push(param);
                        $scanner.take_ws();
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

macro_rules! hdr_list {
    ($scanner:ident => $body:expr) => {{
        let mut hdr_itens = Vec::new();
        $crate::macros::comma_sep!($scanner => {
            hdr_itens.push($body);
        });
        hdr_itens
    }};
}

macro_rules! comma_sep {
    ($scanner:ident => $body:expr) => {{
        $scanner.take_ws();
        $body

        while let Some(b',') = $scanner.peek() {
            $scanner.advance();
            $scanner.take_ws();
            $body
        }
    }};
}

#[macro_export]
macro_rules! filter_map_header {
    ($hdrs:expr, $header:ident) => {
        $hdrs.filter_map(|hdr| {
            if let $crate::headers::Header::$header(v) = hdr {
                Some(v)
            } else {
                None
            }
        })
    };
}

#[macro_export]
macro_rules! find_map_header {
    ($hdrs:expr, $header:ident) => {
        $hdrs.find_map(|hdr| {
            if let $crate::headers::Header::$header(v) = hdr {
                Some(v)
            } else {
                None
            }
        })
    };
}

macro_rules! parse_error {
    ($message:expr) => {{
        Err($crate::error::Error::ParseError($crate::error::SipParserError::new(
            $message,
        )))
    }};
    ($message:expr, $scanner:expr) => {{
        Err($crate::error::Error::ParseError($crate::error::SipParserError::new(
            format!(
                "{} line {} col {}",
                $message,
                $scanner.pos().line(),
                $scanner.pos().col()
            ),
        )))
    }};
}

macro_rules! parse_header {
    ($header:ident, $scanner:ident) => {{
        let Ok(header) = $header::parse($scanner) else {
            return parse_error!(format!("Error parsing '{}' header", $header::NAME), $scanner);
        };
        header
    }};
}

pub(crate) use b_map;
pub(crate) use comma_sep;
pub(crate) use hdr_list;
pub(crate) use parse_header;
pub(crate) use parse_header_param;
pub(crate) use parse_param;

pub(crate) use parse_error;
