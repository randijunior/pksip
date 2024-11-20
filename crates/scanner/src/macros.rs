#[macro_export]
macro_rules! space {
    ($scanner:ident) => {{
        $scanner.read_while($crate::util::is_space);
    }};
}
#[macro_export]
macro_rules! digits {
    ($scanner:ident) => {{
        let range = $scanner.read_while($crate::util::is_digit);

        &$scanner.src[range]
    }};
}
#[macro_export]
macro_rules! read_while {
    ($scanner:expr, $func:expr) => {{
        let range = $scanner.read_while($func);

        &$scanner.src[range]
    }};
}
#[macro_export]
macro_rules! until_byte {
    ($scanner:expr, $byte:expr) => {{
        let range = $scanner.read_while(|b| b != $byte);

        &$scanner.src[range]
    }};
}
#[macro_export]
macro_rules! until_newline {
    ($scanner:ident) => {{
        let range = $scanner.read_while(|b| !$crate::util::is_newline(b));

        &$scanner.src[range]
    }};
}
#[macro_export]
macro_rules! peek_while {
    ($scanner:expr, $func:expr) => {{
        let processed = $scanner.peek_while($func);

        (&$scanner.src[$scanner.idx()..processed])
    }};
}
#[macro_export]
macro_rules! newline {
    ($scanner:ident) => {{
        $scanner.read_while($crate::util::is_newline);
    }};
}
#[macro_export]
macro_rules! alpha {
    ($scanner:ident) => {{
        let range = $scanner.read_while($crate::util::is_alphabetic);

        &$scanner.src[range]
    }};
}


pub use space;
pub use until_byte;
pub use until_newline;
pub use peek_while;
pub use read_while;
pub use digits;
pub use newline;
pub use alpha;