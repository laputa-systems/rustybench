# Development and source map

The repository pins `nightly-2026-09-15` in [`rust-toolchain.toml`](../rust-toolchain.toml).
CI runs Clippy and tests on Linux and macOS.

## Maintainer targets

The [`Makefile`](../Makefile) defines these local workflows:

- `make test` runs `cargo test --locked`.
- `make lint` runs Clippy with warnings denied and does not edit source files.
- `make publish` runs [`tools/bump_minor_release.sh`](../tools/bump_minor_release.sh).
- `make docker-build-arm64`, `make docker-test-arm64`, and
  `make docker-bench-arm64` build the Linux ARM64 image and run its test or
  benchmark command.

`sh tools/test_bump_minor_release.sh` checks the release script's version
comparison and planning functions without publishing.

The repository's supported targets, public names, timing backend, and schema
constraints are summarized in [AGENTS.md](../AGENTS.md).
