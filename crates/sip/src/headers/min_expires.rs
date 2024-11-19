use std::str;

use crate::{parser::Result, scanner::Scanner};

use crate::headers::SipHeader;

/// The `Min-Expires` SIP header.
///
/// The minimum refresh interval supported for soft-state elements managed by that server.
pub struct MinExpires(u32);

impl<'a> SipHeader<'a> for MinExpires {
    const NAME: &'static str = "Min-Expires";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let expires = scanner.read_num()?;

        Ok(MinExpires(expires))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"60";
        let mut scanner = Scanner::new(src);
        let mime_version = MinExpires::parse(&mut scanner).unwrap();

        assert_eq!(mime_version.0, 60);
    }
}
