# rustybench

A small Rust benchmark runner for macOS and Linux. Rustybench requires nightly
Rust 1.100 or newer; the repository pins `nightly-2026-09-15`.

## Install and run

Add Rustybench as a dev-dependency and disable Cargo's default benchmark
harness for each benchmark target:

```toml
[dev-dependencies]
rustybench = "0.3"

[[bench]]
name = "example"
harness = false
```

Add `benches/example.rs` for the `example` target:

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

```sh
cargo bench --bench example
cargo bench --bench example -- --list
cargo bench --bench example -- add
```

The runner accepts command-line options for filtering and output format, plus
`RUSTYBENCH_*` environment settings for sorting, sampling, threads, timing, and
counters. Use `--help` for the full list. Output is human-readable by default; select
`--format terse` for compact output or `--format json` for machine-readable
reports. See the [command reference](docs/commands.md) and [report format](docs/reports.md).

See the [benchmarking guide](docs/benchmarking.md) for registration, input
ownership, counters, and allocation profiling. The [timing guide](docs/timing.md)
explains sample timing and calibration.

For Rust API documentation, see [docs.rs](https://docs.rs/rustybench/0.3.0).
For help, visit the [project repository](https://github.com/laputa-systems/rustybench).

## License

MIT; see [LICENSE](LICENSE).
