#[macro_export]
macro_rules! space {
    ($reader:ident) => {{
        $reader.read_while($crate::util::is_space);
    }};
}
#[macro_export]
macro_rules! digits {
    ($reader:ident) => {{
        let range = $reader.read_while($crate::util::is_digit);

        &$reader.src[range]
    }};
}
#[macro_export]
macro_rules! read_while {
    ($reader:expr, $func:expr) => {{
        let range = $reader.read_while($func);

        &$reader.src[range]
    }};
}
#[macro_export]
macro_rules! until_byte {
    ($reader:expr, $byte:expr) => {{
        let range = $reader.read_while(|b| b != $byte);

        &$reader.src[range]
    }};
}
#[macro_export]
macro_rules! until_newline {
    ($reader:ident) => {{
        let range = $reader.read_while(|b| !$crate::util::is_newline(b));

        &$reader.src[range]
    }};
}
#[macro_export]
macro_rules! peek_while {
    ($reader:expr, $func:expr) => {{
        let processed = $reader.peek_while($func);

        processed
    }};
}
#[macro_export]
macro_rules! newline {
    ($reader:ident) => {{
        $reader.read_while($crate::util::is_newline);
    }};
}
#[macro_export]
macro_rules! alpha {
    ($reader:ident) => {{
        let range = $reader.read_while($crate::util::is_alphabetic);

        &$reader.src[range]
    }};
}

pub use alpha;
pub use digits;
pub use newline;
pub use peek_while;
pub use read_while;
pub use space;
pub use until_byte;
pub use until_newline;
