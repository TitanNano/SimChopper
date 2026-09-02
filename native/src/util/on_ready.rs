use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub(crate) struct OnReady<T>(Option<T>);

impl<T> OnReady<T> {
    pub fn init(&mut self, value: T) {
        self.0 = Some(value);
    }
}

impl<T> Default for OnReady<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> Deref for OnReady<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> DerefMut for OnReady<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
