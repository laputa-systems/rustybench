# Commands and configuration

Pass benchmark executable options after Cargo's `--` separator. The examples
below assume the repository's pinned toolchain is selected.

## Benchmark runner

Run `cargo bench --bench example -- --help` for the runner's options. Positional
filters are regular expressions unless you pass `--exact`. `--skip` excludes
matching names. `--test` runs selected benchmarks once without timing.

The default sample count is 100. If `--sample-size` is omitted, Rustybench
tunes the iterations per sample. With threaded benchmarks, sample count is
rounded up to a multiple of the thread count. Byte rates use decimal units by
default.

`cargo bench` passes `--bench`, which selects a timed run. Without `--bench`,
the runner performs one untimed iteration per benchmark, which is what
`cargo test --benches` expects. `--format json` is printed only for a timed
run. `--nocapture` and `--show-output` are accepted and ignored. `--test` and
`--bench` can also be passed explicitly after Cargo's separator.

Sampling, sorting, and counter settings have `RUSTYBENCH_*` variables.
[src/cli.rs](../src/cli.rs) defines their names and value types. Action, format,
and filter flags have no environment equivalents. CLI values override the
environment.

## Baseline tools

The `rustybench` binary provides baseline collection, comparison, budget checks,
and a Linux syscall diagnostic. Install the tool as described in
[README.md](../README.md), or run `cargo run --bin rustybench --` in this
repository. `rustybench --help` lists all options; subcommand `--help` is not
implemented.

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

Collection combines every schema-1 report emitted by a command, including
reports from multiple benchmark targets. Duplicate benchmark names and invalid
reports fail collection before replacing the baseline.

Every measured command must report the same benchmark inventory exactly once,
with nonzero samples and iterations for each entry. Missing or added entries
fail collection without replacing the baseline. Child diagnostics are relayed
to stderr.

Collection currently assumes Cargo-style argument forwarding; direct benchmark
executables are tracked in [TODO.md](../TODO.md).

Set `RUSTYBENCH_BENCH_RUSTFLAGS` to replace the child benchmark command's
`RUSTFLAGS`. Inherited `RUSTFLAGS` are not kept. This does not change flags
for the `rustybench` tool itself.

## Syscall diagnostic

`rustybench syscalls` exposes a Linux-only Docker and `strace` diagnostic with
defaults of 10 samples and 100 iterations. It requires Docker and `SYS_PTRACE`.
The current command fails at image-build dispatch; its benchmark-name selection
also needs repair. See [TODO.md](../TODO.md). Its output contract is a human
table or diagnostic JSON, separate from benchmark timing.
