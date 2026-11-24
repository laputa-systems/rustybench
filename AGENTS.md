# Rustybench

Rust benchmark runner for macOS and Linux.

The roadmap, [docs/roadmap.md](docs/roadmap.md), holds all current and future work; its Status is the resume point.

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

`make clean` removes the checkout’s ordinary `./target` directory. It refuses a file or symlink at that path.

- Edit: `cargo test --lib alloc_median_follows_allocation_order_not_time_order`;
  Change: `cargo test --test collector`; Full (manual): `make test`.
  Before edits: 102 library tests 0.81 s, 32 tool tests 0.68 s, one collector,
  84 doctests 17.08 s; full cold build and test 139.04 s under shared load.
- Baseline rejection and preservation: `collection_rejects_ambiguous_or_invalid_reports_without_replacing_baseline`
  and `collection_preserves_an_existing_baseline_with_duplicate_names`;
  duplicate comparisons: `comparison_rejects_duplicate_names_in_both_file_roles`.
- Budget ceilings: `check_accepts_candidate_within_all_budget_limits` and the
  metric regression tests. Advisory diff and CLI exit codes: `tests/collector.rs`.
  Comparison text and spread: `comparison_reports_added_removed_and_unstable_benchmarks`
  and `spread_format_distinguishes_stable_and_unstable_runs`.
- JSON schema: `json_round_trip_preserves_report_contract` and compatibility
  tests. Human report headers and sample counts: `tests/collector.rs`.
- Sample median and extrema, allocation accounting: `alloc_median_follows_allocation_order_not_time_order`
  and `max_alloc_bytes_stays_the_sample_peak`. Macro options: `attribute_options_inherit_and_override`,
  `comparisons_and_following_options_register_independently`, and compile-fail doctests.
- `make test` and `make lint` passed on macOS ARM64 with pinned
  nightly 2026-11-09. The empty-target test took 191.75 s and the
  warm repeat took 18.46 s. Clippy took 123.13 s. These observations
  include shared host load. CI runs `make test` on Linux ARM64 and macOS.
- Full installed help and Docker strace execution await separate qualification.
- `make coverage` reports 83.57% lines and 75.28% branches after this audit;
  coverage is informational.
- The fleet lists 17 direct development consumers; check them on public API changes.
- `benches/bench.rs` exercises runner behavior; it needs performance-gate qualification.
