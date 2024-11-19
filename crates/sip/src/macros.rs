macro_rules! space {
    ($scanner:ident) => {{
        $scanner.read_while($crate::util::is_space);
    }};
}

macro_rules! digits {
    ($scanner:ident) => {{
        let range = $scanner.read_while($crate::util::is_digit);

        &$scanner.src[range]
    }};
}

macro_rules! read_while {
    ($scanner:expr, $func:expr) => {{
        let range = $scanner.read_while($func);

        &$scanner.src[range]
    }};
}

macro_rules! until_byte {
    ($scanner:expr, $byte:expr) => {{
        let range = $scanner.read_while(|b| b != $byte);

        &$scanner.src[range]
    }};
}

macro_rules! remaing {
    ($scanner:ident) => {{
        &$scanner.src[$scanner.idx()..]
    }};
}

macro_rules! until_newline {
    ($scanner:ident) => {{
        let range = $scanner.read_while(|b| !$crate::util::is_newline(b));

        &$scanner.src[range]
    }};
}

macro_rules! peek_while {
    ($scanner:expr, $func:expr) => {{
        let processed = $scanner.peek_while($func);

        (&$scanner.src[$scanner.idx()..processed])
    }};
}

macro_rules! newline {
    ($scanner:ident) => {{
        $scanner.read_while($crate::util::is_newline);
    }};
}

macro_rules! alpha {
    ($scanner:ident) => {{
        let range = $scanner.read_while($crate::util::is_alphabetic);

        &$scanner.src[range]
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
        $crate::macros::space!($scanner);
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
                                $crate::macros::space!($scanner);
                                continue;
                            }
                        )*
                        params.set(param.0, param.1.unwrap_or(""));
                        $crate::macros::space!($scanner);
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
        $crate::macros::space!($scanner);
        $body

        while let Some(b',') = $scanner.peek() {
            $scanner.next();
            $crate::macros::space!($scanner);
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
