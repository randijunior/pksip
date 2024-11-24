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
    ($reader:ident) => (
        $crate::macros::parse_param!(
            $reader,
            $crate::headers::parse_header_param,
        )
    );

    ($reader:ident, $($name:ident = $var:expr),*) => (
        $crate::macros::parse_param!(
            $reader,
            $crate::headers::parse_header_param,
            $($name = $var),*
        )
    );
}

macro_rules! parse_param {
    (
        $reader:ident,
        $func:expr,
        $($name:ident = $var:expr),*
    ) =>  {{
        reader::space!($reader);
        match $reader.peek() {
            Some(&b';') => {
                let mut params = $crate::message::Params::new();
                while let Some(&b';') = $reader.peek() {
                        // take ';' character
                        $reader.next();
                        let param = $func($reader)?;
                        $(
                            if param.0 == $name {
                                $var = param.1;
                                reader::space!($reader);
                                continue;
                            }
                        )*
                        params.set(param.0, param.1.unwrap_or(""));
                        reader::space!($reader);
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
    ($reader:ident => $body:expr) => {{
        let mut hdr_itens = Vec::new();
        $crate::macros::comma_sep!($reader => {
            hdr_itens.push($body);
        });
        hdr_itens
    }};
}

macro_rules! comma_sep {
    ($reader:ident => $body:expr) => {{
        reader::space!($reader);
        $body

        while let Some(b',') = $reader.peek() {
            $reader.next();
            reader::space!($reader);
            $body
        }
    }};
}

macro_rules! sip_parse_error {
    ($message:expr) => {{
        Err(crate::parser::SipParserError::from($message))
    }};
}

pub(crate) use b_map;
pub(crate) use comma_sep;
pub(crate) use hdr_list;
pub(crate) use parse_header_param;
pub(crate) use parse_param;

pub(crate) use sip_parse_error;
