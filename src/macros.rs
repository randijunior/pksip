macro_rules! space {
    ($reader:ident) => {{
        $reader.read_while(crate::util::is_space)?;
    }};
}

macro_rules! digits {
    ($reader:ident) => {{
        let (start, end) = $reader.read_while(crate::util::is_digit)?;

        &$reader.input[start..end]
    }};
}

macro_rules! read_while {
    ($reader:expr, $func:expr) => {{
        let (start, end) = $reader.read_while($func)?;

        &$reader.input[start..end]
    }};
}

macro_rules! until_byte {
    ($reader:expr, $byte:expr) => {{
        let (start, end) = $reader.read_while(|b| b != $byte)?;

        &$reader.input[start..end]
    }};
}

macro_rules! tag {
    ($reader:expr, $tag:expr) => {{
        let (start, end) = $reader.tag($tag)?;

        &$reader.input[start..end]
    }};
}

macro_rules! until_newline {
    ($reader:ident) => {{
        let (start, end) = $reader.read_while(|b| !crate::util::is_newline(b))?;

        &$reader.input[start..end]
    }};
}

macro_rules! newline {
    ($reader:ident) => {{
        $reader.read_while(crate::util::is_newline)?;
    }};
}

macro_rules! alpha {
    ($reader:ident) => {{
        let (start, end) = $reader.read_while(crate::util::is_alphabetic)?;

        &$reader.input[start..end]
    }};
}

macro_rules! b_map {
    ($($f:expr,)*) => ([
      $($f != 0,)*
    ])
}

macro_rules! sip_parse_error {
    ($message:expr) => {{
        Err(crate::parser::SipParserError {
            message: $message.to_string(),
        })
    }};
}

pub(crate) use alpha;
pub(crate) use b_map;
pub(crate) use digits;
pub(crate) use newline;
pub(crate) use read_while;
pub(crate) use sip_parse_error;
pub(crate) use space;
pub(crate) use tag;
pub(crate) use until_byte;
pub(crate) use until_newline;
