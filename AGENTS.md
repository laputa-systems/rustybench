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
- The timer is `std::time::Instant` with the full fence boundary.
- JSON reports and baselines use schema 1; additive fields stay optional, and breaking field changes require a schema bump.
- Baseline files are JSON.

## Documentation map

- Registration, inputs, counters, and allocation profiling: [docs/benchmarking.md](docs/benchmarking.md)
- Runner and tool commands, flags, and environment: [docs/commands.md](docs/commands.md)
- JSON reports and baseline schema: [docs/reports.md](docs/reports.md)
- Timer behavior and precision: [docs/timing.md](docs/timing.md)
- Source map and maintainer workflows: [docs/development.md](docs/development.md)

## Verification

- `make test` and `make lint` passed on macOS ARM64 with pinned
  nightly 2026-11-09. The empty-target test took 191.75 s and the
  warm repeat took 18.46 s. Clippy took 123.13 s. These observations
  include shared host load. CI runs `make test` on Linux ARM64 and macOS.
- Macro and compile-fail tests cover registration. Library and collector
  tests cover counters, allocation, builder options and runner execution.
- CLI and collector tests cover option handling, selected exits, and the
  JSON report and baseline contracts. Full installed help and Docker
  strace execution await separate qualification.
- The fleet lists 17 direct development consumers in the owning report.
  Check affected consumers when the public API changes.
- `benches/bench.rs` exercises representative runner behavior. The
  workload needs qualification before use as a runtime performance gate.
