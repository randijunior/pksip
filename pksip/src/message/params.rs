use std::fmt;

/// A parameter.
///
/// This struct represents a parameter in a SIP message,
/// consisting of a name and an optional value.
///
/// # Examples
///
/// ```
/// use pksip::message::Param;
///
/// let param: Param = "param=value".try_into().unwrap();
///
/// assert_eq!(param.name, "param");
/// assert_eq!(param.value, Some("value"));
/// ```
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Param<'a> {
    /// The parameter name.
    pub name: &'a str,

    /// The parameter optional value
    pub value: Option<&'a str>,
}

impl<'a> TryFrom<&'a str> for Param<'a> {
    type Error = crate::error::Error;

    fn try_from(s: &'a str) -> std::result::Result<Self, Self::Error> {
        let mut ctx = crate::parser::ParseCtx::new(s.as_bytes());

        ctx.parse_param()
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
/// A collection of SIP parameters.
///
/// A parameter takes the form `name=value` and can appear in a SIP message
/// as either a URI parameter or a header parameter.
pub struct Params<'p>(Vec<Param<'p>>);

impl<'p> Params<'p> {
    /// Creates an empty `Params` list.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns the number of parameters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets the value of a parameter by name.
    ///
    /// Returns the value associated with the given name, if it exists.
    pub fn get(&self, name: &'p str) -> Option<Option<&str>> {
        self.0
            .iter()
            .find(|&&Param { name: p_name, .. }| p_name == name)
            .map(|&Param { value, .. }| value)
    }

    /// Returns an iterator over the parameters.
    pub fn iter(&self) -> impl Iterator<Item = &Param> {
        self.0.iter()
    }

    /// Pushes a name-value parameter pair.
    pub fn push(&mut self, param: Param<'p>) {
        self.0.push(param)
    }

    /// Checks if the parameter list is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Params<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for Param { name, value } in &self.0 {
            write!(f, "{}={}", name, value.unwrap_or(""))?;
        }
        Ok(())
    }
}

impl<'a, const N: usize> From<[(&'a str, &'a str); N]> for Params<'a> {
    fn from(params: [(&'a str, &'a str); N]) -> Self {
        Self(
            params
                .map(|(name, value)| Param {
                    name,
                    value: value.into(),
                })
                .to_vec(),
        )
    }
}
