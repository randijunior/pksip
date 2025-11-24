//! Text scanning with the `Scanner` type.

use std::fmt::Display;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Default, Hash)]
#[repr(transparent)]
/// A thread-safe reference-counted string type.
/// This type is used throughout the application for sharing string data between
/// threads.
pub struct ArcStr(std::sync::Arc<str>);

impl std::ops::Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ArcStr {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl PartialEq<str> for ArcStr {
    fn eq(&self, other: &str) -> bool {
        &self[..] == other
    }
}

impl PartialEq<&str> for ArcStr {
    fn eq(&self, other: &&str) -> bool {
        &self[..] == *other
    }
}

impl Display for ArcStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<&str> for ArcStr {
    fn from(s: &str) -> Self {
        Self(std::sync::Arc::from(s))
    }
}
