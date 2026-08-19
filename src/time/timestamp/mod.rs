#[cfg(feature = "quanta-timer")]
use quanta::Instant;
#[cfg(not(feature = "quanta-timer"))]
use std::time::Instant;
#[cfg(feature = "quanta-timer")]
use std::num::NonZeroU8;

use crate::time::{FineDuration, fence};

/// A measurement timestamp for the selected monotonic clock backend.
///
/// The default backend is [`std::time::Instant`]. The `quanta-timer` feature
/// swaps only the clock source; the fence boundary around each read remains
/// part of Rustybench's measurement contract in either mode.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Timestamp {
    instant: Instant,
    // Keep a niche in the optional backend so `Option<RawSample>` retains the
    // compact layout used by the thread-pool result buffer.
    #[cfg(feature = "quanta-timer")]
    marker: NonZeroU8,
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
        Self {
            instant,
            #[cfg(feature = "quanta-timer")]
            marker: NonZeroU8::new(1).unwrap(),
        }
    }

    /// Returns the later timestamp when the backend can distinguish the two.
    ///
    /// `quanta::Instant` intentionally does not expose an ordering because its
    /// duration operations saturate when a clock source moves backwards. The
    /// benchmark runner only needs the latest end timestamp for external-time
    /// accounting, so preserving the first value on an indistinguishable or
    /// backwards comparison is sufficient and keeps both backends equivalent.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        if other.duration_since(self).is_zero() {
            self
        } else {
            other
        }
    }
}
