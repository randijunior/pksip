use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Params<'a> {
    pub(crate) inner: HashMap<&'a str, &'a str>,
}

impl<'a> From<HashMap<&'a str, &'a str>> for Params<'a> {
    fn from(value: HashMap<&'a str, &'a str>) -> Self {
        Self { inner: value }
    }
}

impl<'a> Params<'a> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn set(&mut self, k: &'a str, v: &'a str) -> Option<&str> {
        self.inner.insert(k, v)
    }
    pub fn get(&self, k: &'a str) -> Option<&&str> {
        self.inner.get(k)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
