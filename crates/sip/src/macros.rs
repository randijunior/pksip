macro_rules! space {
    ($bytes:ident) => {{
        $bytes.read_while(crate::util::is_space);
    }};
}

macro_rules! digits {
    ($bytes:ident) => {{
        let range = $bytes.read_while(crate::util::is_digit);

        &$bytes.src[range]
    }};
}

macro_rules! read_while {
    ($bytes:expr, $func:expr) => {{
        let range = $bytes.read_while($func);

        &$bytes.src[range]
    }};
}

macro_rules! until_byte {
    ($bytes:expr, $byte:expr) => {{
        let range = $bytes.read_while(|b| b != $byte);

        &$bytes.src[range]
    }};
}

macro_rules! remaing {
    ($bytes:ident) => {{
        &$bytes.src[$bytes.idx()..]
    }};
}

macro_rules! until_newline {
    ($bytes:ident) => {{
        let range = $bytes.read_while(|b| !crate::util::is_newline(b));

        &$bytes.src[range]
    }};
}

macro_rules! peek_while {
    ($bytes:expr, $func:expr) => {{
        let processed = $bytes.peek_while($func);

        (&$bytes.src[$bytes.idx()..processed])
    }};
}

macro_rules! newline {
    ($bytes:ident) => {{
        $bytes.read_while(crate::util::is_newline);
    }};
}

macro_rules! alpha {
    ($bytes:ident) => {{
        let range = $bytes.read_while(crate::util::is_alphabetic);

        &$bytes.src[range]
    }};
}

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
    ($bytes:ident) => (
        $crate::macros::parse_param!(
            $bytes,
            $crate::headers::parse_header_param,
        )
    );

    ($bytes:ident, $($name:ident = $var:expr),*) => (
        $crate::macros::parse_param!(
            $bytes,
            $crate::headers::parse_header_param,
            $($name = $var),*
        )
    );
}

macro_rules! parse_param {
    (
        $bytes:ident,
        $func:expr,
        $($name:ident = $var:expr),*
    ) =>  {{
        $crate::macros::space!($bytes);
        match $bytes.peek() {
            Some(&b';') => {
                let mut params = $crate::uri::Params::new();
                while let Some(&b';') = $bytes.peek() {
                        // take ';' character
                        $bytes.next();
                        let param = $func($bytes)?;
                        $(
                            if param.0 == $name {
                                $var = param.1;
                                $crate::macros::space!($bytes);
                                continue;
                            }
                        )*
                        params.set(param.0, param.1.unwrap_or(""));
                        $crate::macros::space!($bytes);
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
    ($bytes:ident => $body:expr) => {{
        let mut hdr_itens = Vec::new();
        $crate::macros::parse_comma_separated!($bytes => {
            hdr_itens.push($body);
        });
        hdr_itens
    }};
}

macro_rules! parse_comma_separated {
    ($bytes:ident => $body:expr) => {{
        $crate::macros::space!($bytes);
        $body

        while let Some(b',') = $bytes.peek() {
            $bytes.next();
            $crate::macros::space!($bytes);
            $body
        }
    }};
}

macro_rules! sip_parse_error {
    ($message:expr) => {{
        Err(crate::parser::SipParserError::from($message))
    }};
}

pub(crate) use alpha;
pub(crate) use b_map;
pub(crate) use digits;
pub(crate) use newline;
pub(crate) use parse_comma_separated;
pub(crate) use parse_header_list;
pub(crate) use parse_header_param;
pub(crate) use parse_param;
pub(crate) use peek_while;
pub(crate) use read_while;
pub(crate) use remaing;
pub(crate) use sip_parse_error;
pub(crate) use space;
pub(crate) use until_byte;
pub(crate) use until_newline;
