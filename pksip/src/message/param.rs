use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use crate::parser::Parser;
use crate::{Error, Result};

pub(crate) type ParameterRef<'a> = (&'a str, Option<&'a str>);

/// A collection of SIP parameters.
///
/// A parameter takes the form `name=value` and can appear in a SIP message as
/// either a URI parameter or a header parameter.
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Parameters {
    inner: Vec<Parameter>,
}

impl Parameters {
    /// Creates an empty `Parameters`.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Returns the number of elements in the parameters.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Gets the value of a parameter by name.
    ///
    /// Returns the value associated with the given `name`, if it exists.
    pub fn get_named(&self, name: &str) -> Option<&str> {
        self.inner
            .iter()
            .find(|Parameter { name: p_name, .. }| p_name.as_ref() == name)
            .map(|Parameter { value, .. }| value.as_deref())?
    }

    /// Returns an iterator over the parameters.
    pub fn iter(&self) -> impl Iterator<Item = &Parameter> {
        self.inner.iter()
    }

    /// Pushes a new parameter into collection.
    pub fn push(&mut self, param: Parameter) {
        self.inner.push(param)
    }

    /// Checks if the parameter list is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for Parameter { name, value } in &self.inner {
            write!(f, ";{}", name)?;
            if let Some(v) = value {
                write!(f, "={}", v)?;
            }
        }
        Ok(())
    }
}

impl<'a, const N: usize> From<[(&'a str, &'a str); N]> for Parameters {
    fn from(params: [(&'a str, &'a str); N]) -> Self {
        let params = params
            .map(|(name, value)| Parameter::new(name, Some(value)))
            .to_vec();

        Self { inner: params }
    }
}

/// A parameter.
///
/// This struct represents a parameter in a SIP message, consisting of a name
/// and an optional value.
///
/// # Examples
///
/// ```
/// use pksip::message::Parameter;
///
/// let param: Parameter = "param=value".parse().unwrap();
///
/// assert_eq!(param.name(), "param");
/// assert_eq!(param.value(), Some("value"));
/// ```
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Parameter {
    /// The parameter name.
    pub(crate) name: Arc<str>,
    /// The parameter optional value
    pub(crate) value: Option<Arc<str>>,
}

impl Parameter {
    /// Creates a new `Parameter` with the given `name` and optional `value`.
    pub fn new(name: &str, value: Option<&str>) -> Self {
        Self {
            name: name.into(),
            value: value.map(|v| v.into()),
        }
    }

    /// Returns the param `name`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the param `value` if any.
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl From<ParameterRef<'_>> for Parameter {
    #[inline]
    fn from((name, value): ParameterRef) -> Self {
        Self {
            name: name.into(),
            value: value.map(|v| v.into()),
        }
    }
}

impl FromStr for Parameter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Parser::new(s.as_bytes()).parse_ref_param()?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_from_str() {
        let param: Parameter = "param=value".parse().unwrap();
        assert_eq!(param.name(), "param");
        assert_eq!(param.value(), Some("value"));
    }

    #[test]
    fn test_parameters_display() {
        let params = Parameters::from([("param1", "value1"), ("param2", "value2")]);
        assert_eq!(params.to_string(), ";param1=value1;param2=value2");
    }

    #[test]
    fn test_parameters_get_named() {
        let params = Parameters::from([("param1", "value1"), ("param2", "value2")]);
        assert_eq!(params.get_named("param1"), Some("value1"));
        assert_eq!(params.get_named("param3"), None);
    }
}
