macro_rules! space {
    ($reader:ident) => ({
        $reader.read_while(crate::util::is_space)?
    })
}

macro_rules! digits {
    ($reader:ident) => ({
        $reader.read_while(crate::util::is_digit)?
    })
}

macro_rules! newline {
    ($reader:ident) => ({
        $reader.read_while(crate::util::is_newline)?
    })
}

macro_rules! alpha {
    ($reader:ident) => ({
        $reader.read_while(crate::util::is_alphabetic)?
    })
}

macro_rules! next {
    ($reader:ident) => ({
        $reader.read()?
    })
}

macro_rules! peek {
    ($reader:ident) => ({
        $reader.peek()
    })
}


macro_rules! sip_parse_error {
    ($message:expr) => ({
        Err(crate::parser::SipParserError { message: $message.to_string() })
    })
}

pub(crate) use digits;
pub(crate) use newline;
pub(crate) use space;
pub(crate) use alpha;
pub(crate) use next;
pub(crate) use peek;
pub(crate) use sip_parse_error;
