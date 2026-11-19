# Development and source map

[`rust-toolchain.toml`](../rust-toolchain.toml) selects the development
nightly. CI runs Clippy and tests on Linux and macOS with that pin.

The allocator feature gate supports the Rust 1.100 minimum declared in
[`Cargo.toml`](../Cargo.toml). Rust 1.101 and later use the stabilized
allocator API. To check both paths, run the workspace on a Rust 1.100
nightly and on the pinned toolchain. On the pin, use
`RUSTFLAGS=-Dstable_features` and `RUSTDOCFLAGS=-Dstable_features` to
check that the compatibility gate stays inactive.

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
