# Rustybench

Rust benchmark runner for macOS and Linux.

## Build, test, and check

- Build: `cargo build --workspace --locked`
- Test: `cargo test --workspace --locked`
- Check: `cargo check --workspace --all-targets --locked`
- Smoke check: `cargo bench --bench bench -- --list`

## Invariants

- Supported targets are macOS and Linux.
- The crate and runner are named `rustybench` and `Rustybench`; renamed dependencies must set `crate = ...` on the benchmark attribute.
- The default timer is `std::time::Instant` with the full fence boundary; `quanta-timer` is experimental.
- JSON reports and baselines use schema 1; additive fields stay optional, and breaking field changes require a schema bump.
- Baseline files are JSON.

## Documentation map

- Registration, inputs, counters, and allocation profiling: [docs/benchmarking.md](docs/benchmarking.md)
- Runner and tool commands, flags, and environment: [docs/commands.md](docs/commands.md)
- JSON reports and baseline schema: [docs/reports.md](docs/reports.md)
- Timer behavior and precision: [docs/timing.md](docs/timing.md)
- Source map and maintainer workflows: [docs/development.md](docs/development.md)
