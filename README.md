# rustybench

Rustybench is a benchmark runner for Rust on macOS and Linux. It measures
functions registered with `#[rustybench::bench]`, reports time and allocation
statistics, and can collect JSON baselines for later comparison.

Rustybench requires nightly Rust 1.100 or newer. Install the latest nightly
with `rustup toolchain install nightly`, then use `cargo +nightly` for commands
that build or run your benchmarks.

## Add a benchmark

Add Rustybench as a development dependency. Disable Cargo's default benchmark
harness for each target:

```toml
[dev-dependencies]
rustybench = "0.4"

[[bench]]
name = "example"
harness = false
```

Create `benches/example.rs`:

```rust
use rustybench::black_box;

fn main() {
    rustybench::main();
}

#[rustybench::bench]
fn add() -> i32 {
    black_box(1) + black_box(42)
}
```

Run the benchmark, list its registered names, or select names with a regular
expression:

```sh
cargo +nightly bench --bench example
cargo +nightly bench --bench example -- --list
cargo +nightly bench --bench example -- add
```

Pass Rustybench options after Cargo's `--`. For example, add `-- --help` to
print the runner options or `-- --format json` to emit JSON. Run one selected
benchmark without timing it with `-- --test`.

## Benchmark functions and groups

Use a function without arguments for a single operation. Return a value when
the result should be used but its destruction should stay outside the timed
operation. Rustybench black-boxes returned values and drops them after the
sample loop.

Use `Bencher` when you need setup, generated inputs, or control over ownership:

```rust
#[rustybench::bench]
fn sort(bencher: rustybench::Bencher) {
    bencher
        .with_inputs(|| vec![3, 1, 2])
        .bench_values(|mut values| values.sort_unstable());
}
```

`with_inputs` creates inputs outside the timed operation. `bench_values` gives
each iteration an owned input. Use `bench_refs` when the operation should
borrow or mutate an input by reference. Use `bench_local` to run on the current
thread. `bench` runs on the configured benchmark threads.

Use `black_box` on inputs that the compiler could otherwise specialize or
optimize away. Rustybench black-boxes returned values. Call
`rustybench::black_box_drop(value)` when you want to discard an output
explicitly. To measure destruction, include `drop(value)` in the operation.

Use `#[rustybench::bench_group]` on a module to apply options to its benchmarks.
Group options cascade to nested groups and benchmarks. A benchmark's default
name is its module path and function name. Set `name` to change the leaf name.

Use `args` to run a function with several runtime values. Use `types` and
`consts` to run generic benchmarks with several types or const parameters:

```rust
const SIZES: &[usize] = &[8, 64, 1024];

#[rustybench::bench(args = SIZES)]
fn process(size: usize) {
    let values = vec![0; size];
    rustybench::black_box(values);
}
```

See the [`bench` attribute reference](https://docs.rs/rustybench/0.4.0/rustybench/attr.bench.html)
for all benchmark and group options, including thread counts and sample limits.

## Count work and allocations

Counters report work per iteration, such as bytes, characters, cycles, or
items. Set a fixed counter on the attribute or `Bencher`. Use `input_counter`
when the count depends on generated input. Rustybench uses counters to report
throughput:

```rust
use rustybench::{Bencher, counter::ItemsCount};

#[rustybench::bench]
fn sort(bencher: Bencher) {
    bencher
        .with_inputs(|| vec![3, 1, 2])
        .input_counter(|values| ItemsCount::new(values.len()))
        .bench_values(|mut values| values.sort_unstable());
}
```

To profile allocations made by the benchmark executable, install
`AllocProfiler` as its global allocator:

```rust
use rustybench::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();
```

Allocation profiling adds work to the measured process. Keep allocator and
profiler settings the same when comparing runs. Allocations on threads outside
Rustybench's control are not counted.

## Run and tune benchmarks

Runner options and matching environment variables are:

| Option | Environment variable | Purpose |
| --- | --- | --- |
| `--sample-count` | `RUSTYBENCH_SAMPLE_COUNT` | Number of statistical samples. Defaults to 100. |
| `--sample-size` | `RUSTYBENCH_SAMPLE_SIZE` | Iterations per sample. Rustybench tunes this when omitted. |
| `--threads` | `RUSTYBENCH_THREADS` | Thread counts, as a comma-separated list. |
| `--min-time` | `RUSTYBENCH_MIN_TIME` | Minimum time spent per benchmark, in seconds. |
| `--max-time` | `RUSTYBENCH_MAX_TIME` | Maximum time spent per benchmark, in seconds. |
| `--skip-ext-time` | `RUSTYBENCH_SKIP_EXT_TIME` | Exclude time outside the benchmark closure from time limits. |
| `--sort` | `RUSTYBENCH_SORT` | Sort by `kind`, `name`, or `location`. |
| `--sortr` | `RUSTYBENCH_SORTR` | Sort in reverse by `kind`, `name`, or `location`. |
| `--items-count` | `RUSTYBENCH_ITEMS_COUNT` | Set the item throughput count. |
| `--bytes-count` | `RUSTYBENCH_BYTES_COUNT` | Set the byte throughput count. |
| `--bytes-format` | `RUSTYBENCH_BYTES_FORMAT` | Show byte rates in `decimal` or `binary` units. |
| `--chars-count` | `RUSTYBENCH_CHARS_COUNT` | Set the character throughput count. |
| `--cycles-count` | `RUSTYBENCH_CYCLES_COUNT` | Set the cycle throughput count. |

Command-line values override environment values. Positional filters select
benchmark names with regular expressions. Pass `--exact` for exact name
matches, or use `--skip FILTER` to exclude matching names. `--ignored` runs
only ignored benchmarks. `--include-ignored` runs ignored and regular
benchmarks.

Rustybench takes each sample with a monotonic clock and a start and end fence.
The default clock is `std::time::Instant`. Rustybench calibrates loop and
allocation tally overhead and subtracts that overhead from sample results.
Very short operations can still be affected by timer precision and measurement
overhead. Increase `sample_size` when a sample does too little work.

Input generation with `with_inputs` happens outside the timed operation.
Returned values are retained until sample cleanup, so their destruction is not
included in operation timing. Use `skip_ext_time` to exclude time outside the
benchmark closure when Rustybench applies minimum and maximum time limits.

## Read reports and compare baselines

Select JSON output with:

```sh
cargo +nightly bench --bench example -- --format json
```

The benchmark report uses schema 1. Its `benchmarks` array contains each
benchmark's name, median time in nanoseconds, allocation counts and bytes,
maximum live allocation counts and bytes, sample count, and iteration count.
Reports may also include process resource measurements. Linux can report CPU
time, context switches, page faults, and memory snapshots. Unsupported or
unavailable measurements stay marked as unavailable rather than zero. Older
schema-1 reports without resource fields remain readable.

Install the `rustybench` command-line tool for baseline collection, comparison,
and budget checks:

```sh
cargo +nightly install rustybench
rustybench baseline --root . -- cargo +nightly bench --bench example
rustybench diff BASELINE CANDIDATE
rustybench check BUDGET CANDIDATE
```

`baseline` runs one warmup and three measured runs by default. It reads JSON
benchmark output and writes a schema-1 baseline JSON file. `diff` compares two
baseline files. `check` treats the first file as metric ceilings and requires
both files to contain the same benchmark names. Use `rustybench --help` and
`rustybench baseline --help` for options.

On Linux, `rustybench syscalls` runs a Docker and `strace` diagnostic. It needs
Docker and permission to use `SYS_PTRACE`. It reports syscall information, not
benchmark timing.

## More documentation

- [Rust API reference](https://docs.rs/rustybench/0.4.0)
- [Benchmarking guide](https://github.com/laputa-systems/rustybench/blob/main/docs/benchmarking.md)
- [Command reference](https://github.com/laputa-systems/rustybench/blob/main/docs/commands.md)
- [JSON reports and baselines](https://github.com/laputa-systems/rustybench/blob/main/docs/reports.md)
- [Timing details](https://github.com/laputa-systems/rustybench/blob/main/docs/timing.md)
- [Project repository and issue tracker](https://github.com/laputa-systems/rustybench)

## License

MIT; see [LICENSE](LICENSE).
