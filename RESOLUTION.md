# Rustybench timing resolution and mechanics

This document describes how Rustybench measures elapsed time, what its reported
resolution means, and what we learned from the platform-specific timer and
low-overhead fence experiments. The current implementation deliberately uses
the conservative path because the alternatives did not produce a reliable
end-to-end improvement.

## Current implementation

Rustybench uses `std::time::Instant` as its internal monotonic timestamp on the
supported macOS and Linux targets. `Instant` delegates to the platform's
monotonic operating-system clock; Rust's current documentation lists
`CLOCK_UPTIME_RAW` on Darwin and `CLOCK_MONOTONIC` on Unix.

The internal [`Timestamp`](src/time/timestamp/mod.rs) preserves a conservative
ordering boundary around each sample:

```text
start: SeqCst hardware/compiler fence -> Instant::now() -> compiler fence
end:   compiler fence -> Instant::now() -> SeqCst hardware/compiler fence
```

The hardware fences are implemented by [`time::fence::full_fence`](src/time/fence.rs).
The compiler fences also include a no-op inline-assembly barrier on supported
architectures. This keeps LLVM from moving benchmark work across the boundary
and provides the conservative hardware ordering originally used by Rustybench
and Divan.

The measured interval is converted to [`FineDuration`](src/time/fine_duration.rs),
whose internal unit is picoseconds. `Instant` supplies nanosecond-scale values;
the extra decimal places are an arithmetic/output unit, not additional clock
resolution.

## Resolution is not read cost

Rustybench prints `Timer precision` during tuning. The value is the smallest
non-zero interval observed by repeatedly taking a `Timestamp::start()` and
`Timestamp::end()` in immediate succession. It is implemented by
`time::timer::measure_precision`.

That value is better described as the **minimum observed measurement interval**
than as the hardware clock's resolution. It includes:

- the start fence sequence;
- the clock read;
- the end fence sequence;
- timestamp subtraction and conversion;
- cache, instruction-pipeline, and operating-system effects.

On the development macOS/aarch64 host, the observed value is about **41 ns**.
This does not mean that every call costs 41 ns, nor that the underlying clock
has only 41 ns resolution. It is the smallest non-zero result of the complete
measurement boundary.

An empty sample can therefore quantize to zero or one clock interval. A sample
with enough work averages the boundary cost over its `sample_size` iterations
and is more stable.

## What is timed

Rustybench measures a sample, not an individual invocation boundary:

```text
sample start
  repeated benchmark operation, sample_size times
sample end
```

Input generation is outside the timed operation when using the input-owning
Bencher methods. Deferred drops are cleaned up after timing so destruction is
not accidentally charged to the operation. Threaded samples use the same
timestamp boundary around coordinated work.

The timestamp pair is paid once per sample and amortized across `sample_size`.
With `sample_size = 1`, the boundary can be a substantial fraction of a tiny
operation. With larger samples, it contributes less to each iteration.

## Calibrated overhead

`time::timer::bench_overheads` measures overhead once and caches it. The
sample-loop calibration runs 100 samples of 10,000 empty black-box loop
iterations. It divides the minimum observed sample by 10,000 and uses that as
the per-iteration loop overhead.

Allocation profiling has separate calibrations for allocation, deallocation,
and reallocation tally operations. Those costs are subtracted according to the
number of recorded operations in the sample.

Calibration reduces bias in reported operation timings; it does not make the
measurement boundary free. It also does not remove quantization from very short
samples.

## Why the platform-specific fast path was removed

We evaluated two changes:

1. Replace `Instant` with direct nanosecond OS calls:
   - macOS: `clock_gettime_nsec_np(CLOCK_UPTIME_RAW)`;
   - Linux: `clock_gettime(CLOCK_MONOTONIC)`.
2. Remove the hardware `SeqCst` fences and retain compiler fences only.

The direct raw-clock implementation preserved the same clock semantics and
passed the macOS and ARM64 Linux suites. It did not produce a reliable
representative-suite improvement. One macOS comparison had measured suite
times of:

```text
previous: 77.6 ms, 73.0 ms, 72.9 ms
raw clock: 74.9 ms, 73.2 ms, 71.6 ms
```

The distributions overlap, and process startup, scheduler noise, and workload
variation dominate the likely wrapper/conversion difference.

The compiler-only fence path reduced some ignored harness probes, but the
effect did not survive as a dependable dogfood-suite improvement. In the
three-run named-mode comparison:

| Mode | Suite wall times | Sum of benchmark medians |
| --- | --- | ---: |
| conservative path | 76.2 ms, 75.5 ms, 73.2 ms | 234 µs |
| compiler-only/raw experiment | 85.1 ms, 90.1 ms, 88.0 ms | 229 µs |

The benchmark-median difference is small and the suite wall time moved in the
opposite direction. Harness-only results also overlapped. More importantly,
compiler fences constrain LLVM but do not provide the same hardware-ordering
contract as the full fence sequence.

We therefore deleted the fast timer mode and the direct raw-clock backend.
There is no `--timer fast` or `RUSTYBENCH_TIMER` setting. Rustybench has one
default timing contract again: the conservative `Instant`/`SeqCst` path.

## Experimental quanta backend

Rustybench also contains an opt-in `quanta-timer` Cargo feature for comparing
the clock source without changing the default contract:

```sh
cargo bench --bench bench --features quanta-timer -- --format json \
  --sample-count 10 --sample-size 1
```

The feature replaces only the timestamp's clock value with `quanta::Instant`.
The full start/end fence sequence, precision measurement, sample-loop
calibration, and allocation calibration remain unchanged. It is therefore an
experimental comparison backend, not a second supported timing contract.

Quanta may select a calibrated CPU counter and performs global calibration on
first use. That can change cross-core behavior, startup cost, and monotonicity
assumptions, so results must be compared on the same host with the same
benchmark options. The default build intentionally continues to use the
conservative `std::time::Instant` path.

### Recorded comparison

On the development macOS/aarch64 host, with one warmup run, three measured
runs, `sample_count = 10`, and `sample_size = 1`, the results were:

| Metric | Default | `quanta-timer` | Change |
| --- | ---: | ---: | ---: |
| Timer precision | 41 ns | 41 ns | unchanged |
| Sum of benchmark medians | 245.2 µs | 240.1 µs | −2.1% |
| Measured suite wall time per run | 75.8 ms | 312.2 ms | +311.8% |
| Harness-control median sum | 2.355 µs | 2.291 µs | −2.7% |

Individual benchmark medians varied from −10.8% to +8.8%, without a
consistent speedup. Each quanta process also paid roughly 200–240 ms of
first-use calibration, which dominated suite wall time. These measurements do
not justify changing the default backend. The `quanta` dependency is optional,
and `default = []` above ensures it is not compiled or linked unless
`--features quanta-timer` is explicitly requested.

## Why not use Mach ticks or CPU counters?

`mach_absolute_time` is a valid monotonic source on macOS, but it returns raw
Mach ticks and requires timebase conversion. Apple recommends the equivalent
nanosecond clock API instead of using raw Mach ticks directly.

On Linux, `CLOCK_MONOTONIC` is commonly served through the vDSO, so bypassing
`Instant` does not necessarily bypass a system call. `CLOCK_MONOTONIC_COARSE`
is faster but intentionally less precise, which is inappropriate for tuning
short benchmarks.

CPU counters such as x86 TSC or the Arm virtual counter can be faster on some
machines, but add contracts around availability, virtualization, cross-core
synchronization, frequency conversion, migration, and serialization. They are
not a safe default for Rustybench's portable macOS/Linux runner.

## Reproducing the current behavior

Print the timer precision and run the dogfood suite with a controlled sample:

```sh
cargo bench --bench bench -- --format json --sample-count 10 --sample-size 1
```

Collect a repeatable host-keyed baseline:

```sh
cargo run --manifest-path Cargo.toml -- baseline \
  --root . \
  --baseline /tmp/rustybench-resolution-baseline.json \
  --warmup 1 \
  --runs 3 \
  --sample-count 10 \
  --sample-size 1 \
  --variant conservative \
  --quiet \
  -- cargo bench --bench bench
```

For Linux/aarch64, use the project Docker path:

```sh
make docker-bench-arm64
```

Compare measurements on the same host with the same allocator configuration,
compiler, sample options, and benchmark selection.

## References

- [Rust `Instant` documentation](https://doc.rust-lang.org/stable/std/time/struct.Instant.html)
- [Apple `mach_absolute_time` documentation](https://developer.apple.com/documentation/kernel/1462446-mach_absolute_time)
- [Linux `clock_gettime` documentation](https://www.man7.org/linux/man-pages/man3/clock_gettime.3.html)
- [Linux vDSO documentation](https://man7.org/linux/man-pages/man7/vdso.7.html)
