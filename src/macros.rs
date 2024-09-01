macro_rules! space {
    ($reader:ident) => {{
        $reader.read_while(crate::util::is_space);
    }};
}

macro_rules! digits {
    ($reader:ident) => {{
        let range = $reader.read_while(crate::util::is_digit);

        &$reader.src[range]
    }};
}

macro_rules! read_while {
    ($reader:expr, $func:expr) => {{
        let range = $reader.read_while($func);

        &$reader.src[range]
    }};
}

macro_rules! read_until_byte {
    ($reader:expr, $byte:expr) => {{
        let range = $reader.read_while(|b| b != $byte);

        &$reader.src[range]
    }};
}

macro_rules! find {
    ($reader:expr, $tag:expr) => {{
        let range = $reader.read_tag($tag)?;

        &$reader.src[range]
    }};
}

macro_rules! until_newline {
    ($reader:ident) => {{
        let range = $reader.read_while(|b| !crate::util::is_newline(b));

        &$reader.src[range]
    }};
}

macro_rules! peek_while {
    ($reader:expr, $func:expr) => {{
        let range = $reader.peek_while($func);

        (&$reader.src[range])
    }};
}

macro_rules! newline {
    ($reader:ident) => {{
        $reader.read_while(crate::util::is_newline);
    }};
}

macro_rules! alpha {
    ($reader:ident) => {{
        let range = $reader.read_while(crate::util::is_alphabetic);

        &$reader.src[range]
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

macro_rules! sip_parse_error {
    ($message:expr) => {{
        Err(crate::parser::SipParserError::from($message))
    }};
}

pub(crate) use alpha;
pub(crate) use b_map;
pub(crate) use digits;
pub(crate) use find;
pub(crate) use newline;
pub(crate) use peek_while;
pub(crate) use read_while;
pub(crate) use sip_parse_error;
pub(crate) use space;
pub(crate) use read_until_byte;
pub(crate) use until_newline;
