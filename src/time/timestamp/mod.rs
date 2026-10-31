use std::time::Instant;

use crate::time::{FineDuration, fence};

/// A measurement timestamp for the monotonic clock backend.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Timestamp {
    instant: Instant,
}

impl Timestamp {
    #[inline(always)]
    pub fn start() -> Self {
        fence::full_fence();
        let value = Self::new(Instant::now());
        fence::compiler_fence();
        value
    }

    #[inline(always)]
    pub fn end() -> Self {
        fence::compiler_fence();
        let value = Self::new(Instant::now());
        fence::full_fence();
        value
    }

    pub fn duration_since(self, earlier: Self) -> FineDuration {
        self.instant.duration_since(earlier.instant).into()
    }

    #[inline(always)]
    fn new(instant: Instant) -> Self {
        Self { instant }
    }
}
