use std::{borrow::Cow, num::NonZeroUsize, time::Duration};

use crate::{counter::CounterSet, time::FineDuration};

#[derive(Clone, Default)]
/// Options parsed from benchmark attributes.
pub struct BenchOptions<'a> {
    /// Number of statistical samples to collect.
    pub sample_count: Option<u32>,

    /// Iterations in each sample.
    pub sample_size: Option<u32>,

    /// Thread counts used for the benchmark.
    pub threads: Option<Cow<'a, [usize]>>,

    /// Counters applied to each iteration.
    pub counters: CounterSet,

    /// Minimum time spent benchmarking.
    pub min_time: Option<Duration>,

    /// Maximum time spent benchmarking.
    pub max_time: Option<Duration>,

    /// Whether external work is excluded from time limits.
    pub skip_ext_time: Option<bool>,

    /// Whether the benchmark is ignored by default.
    pub ignore: Option<bool>,
}

impl<'a> BenchOptions<'a> {
    pub(crate) fn thread_names(&self, path: &str) -> Vec<(NonZeroUsize, String)> {
        let mut counts: Vec<_> = self.threads.as_deref().unwrap_or_default().iter()
            .map(|&count| NonZeroUsize::new(count).unwrap_or_else(crate::util::known_parallelism))
            .collect();
        counts.sort_unstable();
        counts.dedup();
        if counts.is_empty() {
            counts.push(NonZeroUsize::MIN);
        }
        let branches = counts.len() > 1;
        counts.into_iter().map(|count| {
            let name = if branches { format!("{path}::t={count}") } else { path.to_owned() };
            (count, name)
        }).collect()
    }

    /// Overwrites `other` with values set in `self`.
    #[must_use]
    pub(crate) fn overwrite<'b>(&'b self, other: &'b Self) -> Self
    where
        'b: 'a,
    {
        Self {
            sample_count: self.sample_count.or(other.sample_count),
            sample_size: self.sample_size.or(other.sample_size),
            threads: self
                .threads
                .as_deref()
                .or(other.threads.as_deref())
                .map(Cow::Borrowed),
            min_time: self.min_time.or(other.min_time),
            max_time: self.max_time.or(other.max_time),
            skip_ext_time: self.skip_ext_time.or(other.skip_ext_time),
            ignore: self.ignore.or(other.ignore),

            counters: self.counters.overwrite(&other.counters),
        }
    }

    pub(crate) fn validate_sampling(&self) -> Result<(), &'static str> {
        if self.sample_count == Some(0) {
            return Err("sample count must be greater than zero");
        }
        if self.sample_size == Some(0) {
            return Err("sample size must be greater than zero");
        }
        if self.max_time == Some(Duration::ZERO) {
            return Err("maximum time must be greater than zero");
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn min_time(&self) -> FineDuration {
        self.min_time.map(FineDuration::from).unwrap_or_default()
    }

    #[inline]
    pub(crate) fn max_time(&self) -> FineDuration {
        self.max_time
            .map(FineDuration::from)
            .unwrap_or(FineDuration::MAX)
    }
}
