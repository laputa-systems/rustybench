# rustybench

A small benchmark runner for Rust. It supports macOS and Linux and needs
Rust 1.100 or newer (the standard `Allocator` API backs `AllocProfiler`).
The repository pins a nightly toolchain in `rust-toolchain.toml`.

Command line parsing uses `lexopt`, machine-readable output is
`miniserde`-based JSON, and benchmark baselines and diffs are built into the
`rustybench` binary rather than an external CLI stack.

## Usage

Add the crate as a dev-dependency and disable Cargo's default bench harness:

```toml
[dev-dependencies]
rustybench = { path = "../rustybench" }

[[bench]]
name = "example"
harness = false
```

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
cargo bench --bench example -- add        # regex filter
cargo bench --bench example -- --list
cargo bench --bench example -- --format json
```

`AGENTS.md` documents the attribute options, `Bencher` input ownership,
counters, allocation profiling, the JSON report schema, and the `baseline`
and `diff` subcommands. `RESOLUTION.md` explains the timing mechanics.

## Make targets

- `make test` runs `cargo test --locked`.
- `make lint` runs `cargo fmt` and `cargo clippy --fix`.
- `make publish` runs `tools/bump_minor_release.sh`.
- `make docker-build-arm64`, `docker-test-arm64`, `docker-bench-arm64` build
  and run the tests and benchmarks in a Linux arm64 container.

## License

MIT
