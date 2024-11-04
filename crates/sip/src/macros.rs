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

macro_rules! read_until_byte {
    ($bytes:expr, $byte:expr) => {{
        let range = $bytes.read_while(|b| b != $byte);

        &$bytes.src[range]
    }};
}

macro_rules! remaing {
    ($bytes:ident) => {{
        let range = $bytes.read_while(|_| true);

        &$bytes.src[range]
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
        let range = $bytes.peek_while($func);

        (&$bytes.src[range])
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

macro_rules! parse_param {
    ($bytes:ident) => (
        parse_param!($bytes,)
    );

    ($bytes:ident, $($name:ident = $var:expr),*) => (
         {
            crate::macros::space!($bytes);
            if let Some(&b';') = $bytes.peek() {
                let mut params = crate::uri::Params::new();
                while let Some(&b';') = $bytes.peek() {
                    // take ';' character
                    $bytes.next();
                    let param = crate::message::headers::parse_param($bytes)?;
                    $(
                        if param.0 == $name {
                            $var = param.1;
                            continue;
                        }
                    )*
                    params.set(param.0, param.1.unwrap_or(""));
                }
                if params.is_empty() {
                    None
                } else {
                    Some(params)
                }
            } else {
                None
            }

         }
    );
}

macro_rules! parse_auth_param {
    ($bytes: expr) => {{
        if $bytes.peek() == Some(&b'=') {
            $bytes.next();
            match $bytes.peek() {
                Some(&b'"') => {
                    $bytes.next();
                    let value = crate::macros::read_until_byte!($bytes, &b'"');
                    $bytes.next();
                    Some((std::str::from_utf8(value)?))
                }
                Some(_) => {
                    let value = crate::macros::read_while!(
                        $bytes,
                        crate::token::is_token
                    );
                    Some(unsafe { std::str::from_utf8_unchecked(value) })
                }
                None => None,
            }
        } else {
            None
        }
    }};
}

macro_rules! parse_header_list {
    ($bytes:ident => $body:expr) => {{
        let mut hdr_itens = Vec::new();
        crate::macros::parse_comma_separated_header!($bytes => {
            hdr_itens.push($body);
        });
        hdr_itens
    }};
}

macro_rules! parse_comma_separated_header {
    ($bytes:ident => $body:expr) => {{
        crate::macros::space!($bytes);
        $body

        while let Some(b',') = $bytes.peek() {
            $bytes.next();
            crate::macros::space!($bytes);
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
pub(crate) use parse_auth_param;
pub(crate) use parse_comma_separated_header;
pub(crate) use parse_header_list;
pub(crate) use parse_param;
pub(crate) use peek_while;
pub(crate) use read_until_byte;
pub(crate) use read_while;
pub(crate) use remaing;
pub(crate) use sip_parse_error;
pub(crate) use space;
pub(crate) use until_newline;
