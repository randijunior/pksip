use crate::{error::Result, headers::SipHeaderParse, macros::hdr_list, message::Method, parser::ParseCtx};
use itertools::Itertools;
use std::fmt;

/// The `Allow` SIP header.
///
/// Indicates what methods is supported by the `UA`.
///
/// # Examples
///
/// ```
/// # use pksip::headers::Allow;
/// # use pksip::message::Method;
/// let mut allow = Allow::new();
///
/// allow.push(Method::Invite);
/// allow.push(Method::Register);
///
/// assert_eq!("Allow: INVITE, REGISTER", allow.to_string());
/// ```
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Allow(Vec<Method>);

impl Allow {
    /// Creates a empty `Allow` header.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Appends an new `Method`.
    pub fn push(&mut self, method: Method) {
        self.0.push(method);
    }

    /// Gets the `Method` at the specified index.
    pub fn get(&self, index: usize) -> Option<&Method> {
        self.0.get(index)
    }

    /// Returns the number of `SipMethods` in the header.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParse<'a> for Allow {
    const NAME: &'static str = "Allow";
    /*
     * Allow  =  "Allow" HCOLON [Method *(COMMA Method)]
     */
    fn parse(parser: &mut ParseCtx) -> Result<Self> {
        let allow = hdr_list!(parser => {
            let b_method = parser.alpha();

            Method::from(b_method)
        });

        Ok(Allow(allow))
    }
}

impl fmt::Display for Allow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Allow::NAME, self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"INVITE, ACK, OPTIONS, CANCEL, BYE\r\n";
        let mut scanner = ParseCtx::new(src);
        let allow = Allow::parse(&mut scanner).unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");

        assert_eq!(allow.get(0), Some(&Method::Invite));
        assert_eq!(allow.get(1), Some(&Method::Ack));
        assert_eq!(allow.get(2), Some(&Method::Options));
        assert_eq!(allow.get(3), Some(&Method::Cancel));
        assert_eq!(allow.get(4), Some(&Method::Bye));
        assert_eq!(allow.get(5), None);
    }
}
