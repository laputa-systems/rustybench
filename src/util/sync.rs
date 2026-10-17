//! Synchronization utilities.

use std::ops::{Deref, DerefMut};

/// Makes the wrapped value [`Send`] + [`Sync`] even though it isn't.
pub struct SyncWrap<T> {
    pub value: T,
}

unsafe impl<T> Sync for SyncWrap<T> {}

impl<T> Deref for SyncWrap<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for SyncWrap<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> SyncWrap<T> {
    #[inline]
    pub const unsafe fn new(value: T) -> Self {
        Self { value }
    }
}

