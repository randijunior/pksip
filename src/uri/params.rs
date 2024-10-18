use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Params<'a> {
    pub(crate) inner: HashMap<&'a str, Option<&'a str>>,
}

impl<'a> From<HashMap<&'a str, Option<&'a str>>> for Params<'a> {
    fn from(value: HashMap<&'a str, Option<&'a str>>) -> Self {
        Self { inner: value }
    }
}

impl<'a> Params<'a> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn set(
        &mut self,
        k: &'a str,
        v: Option<&'a str>,
    ) -> Option<Option<&str>> {
        self.inner.insert(k, v)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct UriParams<'a> {
    pub(crate) user: Option<&'a str>,
    pub(crate) method: Option<&'a str>,
    pub(crate) transport: Option<&'a str>,
    pub(crate) ttl: Option<&'a str>,
    pub(crate) lr: Option<&'a str>,
    pub(crate) maddr: Option<&'a str>,
}
