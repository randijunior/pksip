#[macro_export]
macro_rules! space {
    ($reader:ident) => {{
        $reader.read_while($crate::util::is_space);
    }};
}
#[macro_export]
macro_rules! digits {
    ($reader:ident) => {{
        $reader.read_while($crate::util::is_digit)
    }};
}

#[macro_export]
macro_rules! until {
    ($reader:expr, $byte:expr) => {{
        $reader.read_while(|b| b != $byte)
    }};
}
#[macro_export]
macro_rules! until_newline {
    ($reader:ident) => {{
        $reader.read_while(|b| !$crate::util::is_newline(b))
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
        $reader.read_while($crate::util::is_alphabetic)
    }};
}

pub use alpha;
pub use digits;
pub use newline;
pub use space;
pub use until;
pub use until_newline;
