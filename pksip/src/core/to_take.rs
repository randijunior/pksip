//! To Take

use std::ops::Deref;
use std::ops::DerefMut;

pub struct ToTake<'a, T> {
    inner: &'a mut Option<T>,
}

impl<'a, T> ToTake<'a, T> {
    pub fn new(inner: &'a mut Option<T>) -> Self {
        Self { inner }
    }

    pub fn take(self) -> T {
        self.inner.take().unwrap()
    }

    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }
}

impl<T> Deref for ToTake<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<T> DerefMut for ToTake<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}
