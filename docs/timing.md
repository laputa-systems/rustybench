# Timing

Rustybench times each sample with a monotonic clock and a start/end fence
boundary. The default clock is `std::time::Instant`; the optional
`quanta-timer` feature replaces only the clock source and keeps the same fence
sequence. The feature is experimental. See the [timestamp implementation](../src/time/timestamp/mod.rs)
and [fences](../src/time/fence.rs).

The conservative default remains because direct clocks and compiler-only fences
did not show a reliable end-to-end improvement.

## Precision and sample cost

The runner prints `Timer precision` during tuning. It is the smallest non-zero
interval observed around the complete timestamp boundary, not the underlying
clock's guaranteed resolution or the cost of each clock read. It includes
fences, the clock reads, and host scheduling and execution effects. The value
is measured by [`timer_precision`](../src/time/timer.rs).

A sample runs the operation `sample_size` times between one timestamp pair.
That boundary cost is shared across the iterations, so very small samples can
be dominated by measurement overhead. If `sample_size` is omitted, Rustybench
tunes it before collecting samples.

## What is measured

Input setup using the input-owning `Bencher` methods happens outside the timed
operation. Returned values are held until sample cleanup so their destruction
does not enter operation timing. Threaded runs coordinate workers around the
sample boundary. The `skip_ext_time` option excludes time outside the
benchmarked closure when available.

Rustybench calibrates loop and allocation-tally overhead and subtracts it from
sample results. Calibration reduces measurement bias but does not remove the
timestamp boundary or quantization from very short samples. See the [benchmark
implementation](../src/benchmark/mod.rs) and [calibration code](../src/time/timer.rs).
