# Development and source map

The repository pins `nightly-2026-09-15` in [`rust-toolchain.toml`](../rust-toolchain.toml).
CI runs Clippy and tests on Linux and macOS.

## Source map

| Topic | Source |
| --- | --- |
| Public API and macro examples | [`src/lib.rs`](../src/lib.rs) |
| Runtime, registration, and benchmark execution | [`src/rustybench.rs`](../src/rustybench.rs), [`src/benchmark/`](../src/benchmark/) |
| Attribute expansion | [`macros/src/`](../macros/src/) and [`src/entry/`](../src/entry/) |
| CLI parsing and runner configuration | [`src/cli.rs`](../src/cli.rs), [`src/config/`](../src/config/) |
| JSON reports and process resources | [`src/report.rs`](../src/report.rs), [`src/resource.rs`](../src/resource.rs) |
| Timing and sample statistics | [`src/time/`](../src/time/), [`src/stats/`](../src/stats/) |
| Allocation profiling | [`src/alloc.rs`](../src/alloc.rs) |
| Baseline, diff, check, and syscall commands | [`src/bin/rustybench.rs`](../src/bin/rustybench.rs) |
| Dogfood benchmark | [`benches/bench.rs`](../benches/bench.rs) |

## Maintainer targets

The [`Makefile`](../Makefile) defines these local workflows:

- `make test` runs `cargo test --locked`.
- `make lint` runs Clippy with `--fix` and can edit source files.
- `make publish` runs [`tools/bump_minor_release.sh`](../tools/bump_minor_release.sh).
- `make docker-build-arm64`, `make docker-test-arm64`, and
  `make docker-bench-arm64` build the Linux ARM64 image and run its test or
  benchmark command.

The repository's supported targets, public names, timing backend, and schema
constraints are summarized in [AGENTS.md](../AGENTS.md).
