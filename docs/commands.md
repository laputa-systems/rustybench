# Commands and configuration

The benchmark executable accepts positional regular-expression filters and
options after Cargo's `--` separator. Run `cargo bench --bench example -- --help`
to print the current help.

## Benchmark runner

Run `cargo bench --bench example -- --help` for the runner's options. Positional
filters are regular expressions unless you pass `--exact`. `--skip` excludes
matching names. `--test` runs selected benchmarks once without timing.

The default sample count is 100. If `--sample-size` is omitted, Rustybench
tunes the iterations per sample. With threaded benchmarks, sample count is
rounded up to a multiple of the thread count. Byte rates use decimal units by
default.

Cargo harness arguments `--bench`, `--nocapture`, and `--show-output` are
accepted for compatibility. `--test` and `--bench` can also be passed explicitly
after Cargo's separator.

Set the matching `RUSTYBENCH_*` environment variable to configure a runner
option. [src/cli.rs](../src/cli.rs) lists the supported names and value types.

## Baseline tools

The `rustybench` binary provides baseline collection, comparison, budget checks,
and a Linux syscall diagnostic. Use `cargo +nightly install rustybench` to
install it, or run it from this repository with
`cargo run --bin rustybench --`. Its help lists all options.

Collect and compare a baseline:

```sh
rustybench baseline --root . -- cargo bench --bench example
rustybench diff BASELINE CANDIDATE
rustybench check BUDGET CANDIDATE
```

Replace `BASELINE`, `CANDIDATE`, and `BUDGET` with paths to schema-1 baseline
files.

`baseline` runs one warmup and three measured commands by default. It reads JSON
benchmark output, compares with an existing baseline, and writes the new
baseline. Without `--baseline`, it writes a host-keyed file under `benches/`.
Use `--warmup`, `--runs`, `--sample-count`, `--sample-size`, and `--variant` to
adjust collection. `--quiet` hides the comparison table. `--print-path` prints
the selected path and exits. `--fast` runs once with no warmup, one sample, and
one iteration per sample.

`diff` prints a comparison between two baseline files. `check` treats the first
file as a set of absolute metric ceilings and requires both files to contain
exactly the same benchmark names.

Set `RUSTYBENCH_BENCH_RUSTFLAGS` to add Rust flags to the child benchmark
command launched by `baseline`; this does not change flags for the `rustybench`
tool itself.

## Syscall diagnostic

`rustybench syscalls` runs a Linux-only Docker and `strace` diagnostic. It needs
Docker and permission to use `SYS_PTRACE`. It defaults to 10 samples of 100
iterations and emits a human table or diagnostic JSON. It does not report
benchmark timing.
