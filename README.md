# rustybench

Rustybench is a benchmark runner for Rust on macOS and Linux. It measures
functions registered with `#[rustybench::bench]`, reports time and allocation
statistics, and can collect JSON baselines for later comparison.

Version 0.5 requires nightly Rust and is tested with `nightly-2026-09-15`
(Rust 1.100 nightly) and Rust 1.101 nightly (`16d1ffac1`, 2026-10-26).
The repository remains pinned to `nightly-2026-09-15`. Install it with
`rustup toolchain install nightly-2026-09-15`.

## Add a benchmark

Add Rustybench as a development dependency. Disable Cargo's default benchmark
harness for each target:

```toml
[dev-dependencies]
rustybench = "0.5"

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
cargo +nightly-2026-09-15 bench --bench example
cargo +nightly-2026-09-15 bench --bench example -- --list
cargo +nightly-2026-09-15 bench --bench example -- add
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

Use `args` for runtime variants, and `types` or `consts` for generic variants.
See the [benchmarking guide](docs/benchmarking.md) and
[`bench` attribute reference](https://docs.rs/rustybench/0.5.0/rustybench/attr.bench.html)
for examples, thread counts, and sample limits.

## Count work and allocations

Counters report work per iteration, such as bytes, characters, cycles, or
items. Set a fixed counter on the attribute or `Bencher`. Use `input_counter`
when the count depends on generated input. For example,
`.input_counter(|values| rustybench::counter::ItemsCount::new(values.len()))` counts items in an
input collection. Rustybench uses counters to report throughput.

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

Run `cargo +nightly-2026-09-15 bench --bench example -- --help` for runner options.
Sampling, sorting, and counter settings have `RUSTYBENCH_*` variables, defined
in [`src/cli.rs`](src/cli.rs). Command-line values override them. Action, format,
and filter flags have no environment equivalents.

Positional filters select benchmark names with regular expressions. Pass
`--exact` for exact name matches, or use `--skip FILTER` to exclude matching
names. `--ignored` runs only ignored benchmarks. `--include-ignored` runs
ignored and regular benchmarks.

Rustybench uses `std::time::Instant` with start and end fences. It calibrates loop and
allocation tally overhead and subtracts that overhead from sample results.
Very short operations can still be affected by timer precision and measurement
overhead. Increase `sample_size` when a sample does too little work.

Use `skip_ext_time` to exclude external work when applying duration limits.
See [timing](docs/timing.md) for sampling and calibration details.

## Read reports and compare baselines

Select JSON output with:

```sh
cargo +nightly-2026-09-15 bench --bench example -- --format json
```

The schema-1 report contains timing and allocation summaries, with optional
process-resource measurements. [Reports and baselines](docs/reports.md)
defines the metrics, platform support, and current measurement limitations.
Read the allocation-peak definition before using those values as budgets.

Install the `rustybench` command-line tool for baseline collection, comparison,
and budget checks. Collect one benchmark target per command:

```sh
cargo +nightly-2026-09-15 install rustybench --version 0.5.0 --locked
rustybench baseline --root . -- cargo +nightly-2026-09-15 bench --bench example
rustybench diff BASELINE CANDIDATE
rustybench check BUDGET CANDIDATE
```

`baseline` runs one warmup and three measured runs by default. It reads JSON
benchmark output and writes a schema-1 baseline JSON file. `diff` compares two
baseline files. `check` treats the first file as metric ceilings and requires
both files to contain the same benchmark names. Use `rustybench --help` for
options and the [command reference](docs/commands.md) for collection limitations.

The Linux Docker/`strace` diagnostic, `rustybench syscalls`, currently fails at
image-build dispatch. Its repairs are tracked in [TODO.md](TODO.md).

## More documentation

- [Rust API reference](https://docs.rs/rustybench/0.5.0)
- [Benchmarking guide](https://github.com/laputa-systems/rustybench/blob/main/docs/benchmarking.md)
- [Command reference](https://github.com/laputa-systems/rustybench/blob/main/docs/commands.md)
- [JSON reports and baselines](https://github.com/laputa-systems/rustybench/blob/main/docs/reports.md)
- [Timing details](https://github.com/laputa-systems/rustybench/blob/main/docs/timing.md)
- [Project repository and issue tracker](https://github.com/laputa-systems/rustybench)

## License

MIT; see [LICENSE](LICENSE).
