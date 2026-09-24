//! Bounded process-resource measurements associated with benchmark samples.
//!
//! Linux process counters come from `getrusage(RUSAGE_SELF)`, which accounts for
//! the whole benchmark process (including worker threads). Linux memory values
//! are read from `/proc/self/status` and `/proc/self/smaps_rollup` when those
//! files are available. The reads are deliberately bounded because this data is
//! diagnostic evidence, not a general-purpose procfs parser.

#[cfg(target_os = "linux")]
use std::{fs, io::Read};

use miniserde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
const PROC_FILE_LIMIT: usize = 64 * 1024;

/// Whether a resource measurement can be interpreted on the current host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ResourceStatus {
    #[serde(rename = "supported")]
    Supported,
    #[serde(rename = "unsupported")]
    Unsupported,
    #[serde(rename = "not_applicable")]
    NotApplicable,
}

/// A process memory snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemorySnapshot {
    pub status: ResourceStatus,
    pub rss_bytes: Option<u64>,
    pub pss_bytes: Option<u64>,
}

impl MemorySnapshot {
    #[cfg(target_os = "linux")]
    fn unsupported() -> Self {
        Self {
            status: ResourceStatus::NotApplicable,
            rss_bytes: None,
            pss_bytes: None,
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn unsupported() -> Self {
        Self {
            status: ResourceStatus::Unsupported,
            rss_bytes: None,
            pss_bytes: None,
        }
    }
}

/// One process-resource snapshot, before and after a timed sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcessResourceSnapshot {
    pub status: ResourceStatus,
    pub user_cpu_ns: Option<u64>,
    pub system_cpu_ns: Option<u64>,
    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,
    pub minor_page_faults: Option<u64>,
    pub major_page_faults: Option<u64>,
    pub memory: MemorySnapshot,
}

/// Resource deltas and the post-sample memory snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcessResourceDelta {
    pub status: ResourceStatus,
    pub user_cpu_ns: Option<u64>,
    pub system_cpu_ns: Option<u64>,
    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,
    pub minor_page_faults: Option<u64>,
    pub major_page_faults: Option<u64>,
    pub memory: MemorySnapshot,
}

/// Median process-resource values for a benchmark's collected samples.
///
/// Values are optional so reports can distinguish a zero measurement from a
/// metric that is unavailable on the host. The status fields make the reason
/// explicit for unsupported platforms and unavailable Linux procfs data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProcessResourceMetrics {
    pub status: ResourceStatus,
    pub user_cpu_ns: Option<u64>,
    pub system_cpu_ns: Option<u64>,
    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,
    pub minor_page_faults: Option<u64>,
    pub major_page_faults: Option<u64>,
    pub memory_status: ResourceStatus,
    pub rss_bytes: Option<u64>,
    pub pss_bytes: Option<u64>,
}

impl ProcessResourceSnapshot {
    /// Takes one bounded process-resource snapshot.
    pub(crate) fn capture() -> Self {
        #[cfg(target_os = "linux")]
        let usage = process_usage();

        Self {
            status: capture_process_status(
                #[cfg(target_os = "linux")]
                usage,
            ),
            user_cpu_ns: process_user_cpu_ns(
                #[cfg(target_os = "linux")]
                usage,
            ),
            system_cpu_ns: process_system_cpu_ns(
                #[cfg(target_os = "linux")]
                usage,
            ),
            voluntary_context_switches: process_voluntary_context_switches(
                #[cfg(target_os = "linux")]
                usage,
            ),
            involuntary_context_switches: process_involuntary_context_switches(
                #[cfg(target_os = "linux")]
                usage,
            ),
            minor_page_faults: process_minor_page_faults(
                #[cfg(target_os = "linux")]
                usage,
            ),
            major_page_faults: process_major_page_faults(
                #[cfg(target_os = "linux")]
                usage,
            ),
            memory: capture_memory(),
        }
    }

    /// Takes a snapshot without charging procfs parsing allocations to the
    /// benchmark's allocation profile.
    pub(crate) fn capture_unprofiled() -> Self {
        let was_ignored = crate::alloc::IGNORE_ALLOC.get();
        crate::alloc::IGNORE_ALLOC.set(true);
        let snapshot = Self::capture();
        crate::alloc::IGNORE_ALLOC.set(was_ignored);
        snapshot
    }
}

impl ProcessResourceDelta {
    /// Computes monotonic counter deltas and retains the post-sample memory.
    pub(crate) fn between(before: ProcessResourceSnapshot, after: ProcessResourceSnapshot) -> Self {
        let status = match (before.status, after.status) {
            (ResourceStatus::Supported, ResourceStatus::Supported) => ResourceStatus::Supported,
            (ResourceStatus::NotApplicable, _) | (_, ResourceStatus::NotApplicable) => {
                ResourceStatus::NotApplicable
            }
            _ => ResourceStatus::Unsupported,
        };

        Self {
            status,
            user_cpu_ns: delta(before.user_cpu_ns, after.user_cpu_ns),
            system_cpu_ns: delta(before.system_cpu_ns, after.system_cpu_ns),
            voluntary_context_switches: delta(
                before.voluntary_context_switches,
                after.voluntary_context_switches,
            ),
            involuntary_context_switches: delta(
                before.involuntary_context_switches,
                after.involuntary_context_switches,
            ),
            minor_page_faults: delta(before.minor_page_faults, after.minor_page_faults),
            major_page_faults: delta(before.major_page_faults, after.major_page_faults),
            memory: after.memory,
        }
    }
}

/// Summarizes resource samples using the same median convention as timing
/// reports: the middle two values are averaged for an even sample count.
pub(crate) fn summarize(samples: &[ProcessResourceDelta]) -> ProcessResourceMetrics {
    let status = summarize_status(samples.iter().map(|sample| sample.status));
    let memory_status = summarize_status(samples.iter().map(|sample| sample.memory.status));

    ProcessResourceMetrics {
        status,
        user_cpu_ns: median(samples.iter().filter_map(|sample| sample.user_cpu_ns)),
        system_cpu_ns: median(samples.iter().filter_map(|sample| sample.system_cpu_ns)),
        voluntary_context_switches: median(
            samples
                .iter()
                .filter_map(|sample| sample.voluntary_context_switches),
        ),
        involuntary_context_switches: median(
            samples
                .iter()
                .filter_map(|sample| sample.involuntary_context_switches),
        ),
        minor_page_faults: median(samples.iter().filter_map(|sample| sample.minor_page_faults)),
        major_page_faults: median(samples.iter().filter_map(|sample| sample.major_page_faults)),
        memory_status,
        rss_bytes: median(samples.iter().filter_map(|sample| sample.memory.rss_bytes)),
        pss_bytes: median(samples.iter().filter_map(|sample| sample.memory.pss_bytes)),
    }
}

fn delta(before: Option<u64>, after: Option<u64>) -> Option<u64> {
    after?.checked_sub(before?)
}

fn summarize_status(statuses: impl Iterator<Item = ResourceStatus>) -> ResourceStatus {
    let mut saw_not_applicable = false;
    let mut saw_any = false;

    for status in statuses {
        saw_any = true;
        match status {
            ResourceStatus::Supported => return ResourceStatus::Supported,
            ResourceStatus::NotApplicable => saw_not_applicable = true,
            ResourceStatus::Unsupported => {}
        }
    }

    if saw_not_applicable {
        ResourceStatus::NotApplicable
    } else if saw_any {
        ResourceStatus::Unsupported
    } else {
        ResourceStatus::NotApplicable
    }
}

fn median(values: impl Iterator<Item = u64>) -> Option<u64> {
    let mut values: Vec<u64> = values.collect();
    if values.is_empty() {
        return None;
    }

    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[middle])
    } else {
        Some(values[middle - 1].saturating_add(values[middle]) / 2)
    }
}

#[cfg(target_os = "linux")]
fn read_bounded(path: &str) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut bytes = Vec::with_capacity(PROC_FILE_LIMIT.min(4096));
    file.take((PROC_FILE_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .ok()?;

    if bytes.len() > PROC_FILE_LIMIT {
        return None;
    }

    String::from_utf8(bytes).ok()
}

#[cfg(target_os = "linux")]
fn parse_memory_bytes(contents: &str, key: &str) -> Option<u64> {
    contents.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        (fields.next()? == key).then(|| {
            let value = fields.next()?.parse::<u64>().ok()?;
            let unit = fields.next().unwrap_or("bytes");
            match unit {
                "kB" => value.checked_mul(1024),
                "bytes" => Some(value),
                _ => None,
            }
        })?
    })
}

#[cfg(target_os = "linux")]
fn capture_memory() -> MemorySnapshot {
    let Some(status) = read_bounded("/proc/self/status") else {
        return MemorySnapshot::unsupported();
    };

    let rss_bytes = parse_memory_bytes(&status, "VmRSS:");
    let pss_bytes = read_bounded("/proc/self/smaps_rollup")
        .and_then(|smaps| parse_memory_bytes(&smaps, "Pss:"));

    if rss_bytes.is_none() && pss_bytes.is_none() {
        MemorySnapshot::unsupported()
    } else {
        MemorySnapshot {
            status: ResourceStatus::Supported,
            rss_bytes,
            pss_bytes,
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn capture_memory() -> MemorySnapshot {
    MemorySnapshot::unsupported()
}

#[cfg(target_os = "linux")]
fn capture_process_status(usage: Option<ProcessUsage>) -> ResourceStatus {
    if usage.is_some() {
        ResourceStatus::Supported
    } else {
        ResourceStatus::NotApplicable
    }
}

#[cfg(not(target_os = "linux"))]
fn capture_process_status() -> ResourceStatus {
    ResourceStatus::Unsupported
}

#[cfg(target_os = "linux")]
fn process_user_cpu_ns(usage: Option<ProcessUsage>) -> Option<u64> {
    usage.and_then(|usage| timeval_ns(usage.user))
}

#[cfg(not(target_os = "linux"))]
fn process_user_cpu_ns() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn process_system_cpu_ns(usage: Option<ProcessUsage>) -> Option<u64> {
    usage.and_then(|usage| timeval_ns(usage.system))
}

#[cfg(not(target_os = "linux"))]
fn process_system_cpu_ns() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn process_voluntary_context_switches(usage: Option<ProcessUsage>) -> Option<u64> {
    usage.and_then(|usage| nonnegative(usage.voluntary_context_switches))
}

#[cfg(not(target_os = "linux"))]
fn process_voluntary_context_switches() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn process_involuntary_context_switches(usage: Option<ProcessUsage>) -> Option<u64> {
    usage.and_then(|usage| nonnegative(usage.involuntary_context_switches))
}

#[cfg(not(target_os = "linux"))]
fn process_involuntary_context_switches() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn process_minor_page_faults(usage: Option<ProcessUsage>) -> Option<u64> {
    usage.and_then(|usage| nonnegative(usage.minor_page_faults))
}

#[cfg(not(target_os = "linux"))]
fn process_minor_page_faults() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn process_major_page_faults(usage: Option<ProcessUsage>) -> Option<u64> {
    usage.and_then(|usage| nonnegative(usage.major_page_faults))
}

#[cfg(not(target_os = "linux"))]
fn process_major_page_faults() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct ProcessUsage {
    user: Timeval,
    system: Timeval,
    minor_page_faults: CLong,
    major_page_faults: CLong,
    voluntary_context_switches: CLong,
    involuntary_context_switches: CLong,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct Timeval {
    seconds: CLong,
    microseconds: CLong,
}

#[cfg(target_os = "linux")]
type CInt = i32;

#[cfg(target_os = "linux")]
type CLong = i64;

#[cfg(target_os = "linux")]
#[repr(C)]
struct RawTimeval {
    seconds: CLong,
    microseconds: CLong,
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct RawRusage {
    user: RawTimeval,
    system: RawTimeval,
    max_rss: CLong,
    integral_shared_memory_size: CLong,
    integral_unshared_data_size: CLong,
    integral_unshared_stack_size: CLong,
    minor_page_faults: CLong,
    major_page_faults: CLong,
    swapped_out: CLong,
    block_input_operations: CLong,
    block_output_operations: CLong,
    messages_sent: CLong,
    messages_received: CLong,
    signals_received: CLong,
    voluntary_context_switches: CLong,
    involuntary_context_switches: CLong,
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn getrusage(who: CInt, usage: *mut RawRusage) -> CInt;
}

#[cfg(target_os = "linux")]
fn process_usage() -> Option<ProcessUsage> {
    let mut raw = std::mem::MaybeUninit::<RawRusage>::zeroed();
    // SAFETY: `raw` points to writable storage with the exact C layout
    // required by getrusage, and RUSAGE_SELF (0) is valid on Linux.
    let result = unsafe { getrusage(0, raw.as_mut_ptr()) };
    if result != 0 {
        return None;
    }

    // SAFETY: getrusage initialized raw after returning success.
    let raw = unsafe { raw.assume_init() };
    Some(ProcessUsage {
        user: Timeval {
            seconds: raw.user.seconds,
            microseconds: raw.user.microseconds,
        },
        system: Timeval {
            seconds: raw.system.seconds,
            microseconds: raw.system.microseconds,
        },
        minor_page_faults: raw.minor_page_faults,
        major_page_faults: raw.major_page_faults,
        voluntary_context_switches: raw.voluntary_context_switches,
        involuntary_context_switches: raw.involuntary_context_switches,
    })
}

#[cfg(target_os = "linux")]
fn timeval_ns(value: Timeval) -> Option<u64> {
    let seconds = u64::try_from(value.seconds).ok()?;
    let microseconds = u64::try_from(value.microseconds).ok()?;
    seconds
        .checked_mul(1_000_000_000)
        .and_then(|ns| ns.checked_add(microseconds.checked_mul(1_000)?))
}

#[cfg(target_os = "linux")]
fn nonnegative(value: CLong) -> Option<u64> {
    u64::try_from(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_proc_memory_units() {
        let contents = "VmRSS:\t12 kB\nPss: 7 kB\n";
        assert_eq!(parse_memory_bytes(contents, "VmRSS:"), Some(12 * 1024));
        assert_eq!(parse_memory_bytes(contents, "Pss:"), Some(7 * 1024));
        assert_eq!(parse_memory_bytes(contents, "VmSize:"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_unbounded_proc_content() {
        let contents = format!("VmRSS: {} kB\n", "x".repeat(PROC_FILE_LIMIT));
        assert!(contents.len() > PROC_FILE_LIMIT);
        assert_eq!(parse_memory_bytes(&contents, "VmRSS:"), None);
    }

    #[test]
    fn deltas_reject_counter_regressions() {
        let memory = MemorySnapshot {
            status: ResourceStatus::Supported,
            rss_bytes: Some(20),
            pss_bytes: Some(10),
        };
        let before = ProcessResourceSnapshot {
            status: ResourceStatus::Supported,
            user_cpu_ns: Some(30),
            system_cpu_ns: Some(10),
            voluntary_context_switches: Some(4),
            involuntary_context_switches: Some(2),
            minor_page_faults: Some(8),
            major_page_faults: Some(1),
            memory,
        };
        let mut after = before;
        after.user_cpu_ns = Some(20);
        after.system_cpu_ns = Some(15);
        after.memory.rss_bytes = Some(40);
        let delta = ProcessResourceDelta::between(before, after);
        assert_eq!(delta.user_cpu_ns, None);
        assert_eq!(delta.system_cpu_ns, Some(5));
        assert_eq!(delta.memory.rss_bytes, Some(40));
    }

    #[test]
    fn summarizes_each_metric_without_turning_missing_values_into_zero() {
        let memory = MemorySnapshot {
            status: ResourceStatus::Supported,
            rss_bytes: Some(100),
            pss_bytes: Some(50),
        };
        let sample = |user_cpu_ns, rss_bytes| ProcessResourceDelta {
            status: ResourceStatus::Supported,
            user_cpu_ns: Some(user_cpu_ns),
            system_cpu_ns: None,
            voluntary_context_switches: Some(2),
            involuntary_context_switches: None,
            minor_page_faults: Some(3),
            major_page_faults: None,
            memory: MemorySnapshot {
                rss_bytes,
                ..memory
            },
        };

        let metrics = summarize(&[sample(3, Some(100)), sample(1, Some(200))]);
        assert_eq!(metrics.status, ResourceStatus::Supported);
        assert_eq!(metrics.user_cpu_ns, Some(2));
        assert_eq!(metrics.system_cpu_ns, None);
        assert_eq!(metrics.voluntary_context_switches, Some(2));
        assert_eq!(metrics.involuntary_context_switches, None);
        assert_eq!(metrics.rss_bytes, Some(150));
        assert_eq!(metrics.pss_bytes, Some(50));
    }

    #[test]
    fn status_is_explicit_on_every_target() {
        let snapshot = ProcessResourceSnapshot::capture();
        assert!(matches!(
            snapshot.status,
            ResourceStatus::Supported | ResourceStatus::Unsupported | ResourceStatus::NotApplicable
        ));
        assert!(matches!(
            snapshot.memory.status,
            ResourceStatus::Supported | ResourceStatus::Unsupported | ResourceStatus::NotApplicable
        ));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_snapshot_exposes_process_counters() {
        let snapshot = ProcessResourceSnapshot::capture();
        assert_eq!(snapshot.status, ResourceStatus::Supported);
        assert!(snapshot.user_cpu_ns.is_some());
        assert!(snapshot.system_cpu_ns.is_some());
        assert!(snapshot.voluntary_context_switches.is_some());
        assert!(snapshot.involuntary_context_switches.is_some());
        assert!(snapshot.minor_page_faults.is_some());
        assert!(snapshot.major_page_faults.is_some());
    }
}
