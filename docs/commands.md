# Commands and configuration

The benchmark executable accepts positional regular-expression filters and
options after Cargo's `--` separator. Run `cargo bench --bench example -- --help`
to print the current help.

## Benchmark runner

| Option | Effect |
| --- | --- |
| `--test` | Run each selected benchmark once without timing. |
| `--list` | List registered benchmarks. |
| `--skip FILTER` | Exclude names matching a filter. |
| `--exact` | Treat filters as exact names. |
| `--ignored`, `--include-ignored` | Run only ignored benchmarks, or include them with ordinary benchmarks. |
| `--sort` / `--sortr` | Sort by `kind`, `name`, or `location`, ascending or descending. |
| `--sample-count N` | Set the number of samples. |
| `--sample-size N` | Set iterations per sample. |
| `--threads N[,N...]` | Set thread counts; zero means available parallelism. |
| `--min-time SECONDS`, `--max-time SECONDS` | Set the tuning and collection time bounds. |
| `--skip-ext-time[=BOOL]` | Exclude time outside the benchmarked closure when available. |
| `--items-count N`, `--bytes-count N`, `--chars-count N`, `--cycles-count N` | Set throughput counters. |
| `--bytes-format` (decimal or binary) | Choose byte-rate units. |
| `--format` (pretty, terse, or json) | Choose human-readable, compact, or JSON output. |

The default sample count is 100. If `--sample-size` is omitted, Rustybench
tunes the iterations per sample. With threaded benchmarks, sample count is
rounded up to a multiple of the thread count. Byte rates use decimal units by
default.

Cargo harness arguments `--bench`, `--nocapture`, and `--show-output` are
accepted for compatibility. `--test` and `--bench` can also be passed explicitly
after Cargo's separator.

The environment variables for runner options are:

| Setting | Environment variable |
| --- | --- |
| Sort order | `RUSTYBENCH_SORT`, `RUSTYBENCH_SORTR` |
| Samples and iterations | `RUSTYBENCH_SAMPLE_COUNT`, `RUSTYBENCH_SAMPLE_SIZE` |
| Threads | `RUSTYBENCH_THREADS` |
| Time bounds | `RUSTYBENCH_MIN_TIME`, `RUSTYBENCH_MAX_TIME` |
| External time | `RUSTYBENCH_SKIP_EXT_TIME` |
| Throughput counters | `RUSTYBENCH_ITEMS_COUNT`, `RUSTYBENCH_BYTES_COUNT`, `RUSTYBENCH_CHARS_COUNT`, `RUSTYBENCH_CYCLES_COUNT` |
| Byte units | `RUSTYBENCH_BYTES_FORMAT` |

## Baseline tools

The `rustybench` binary provides baseline collection, comparison, budget checks,
and a Linux syscall diagnostic. Install it with `cargo install rustybench`, or
run it from this repository with `cargo run --bin rustybench --`. Its help
lists all options.

Collect and compare a baseline:

```sh
rustybench baseline --root . -- cargo bench --bench example
rustybench diff BASELINE CANDIDATE
rustybench check BUDGET CANDIDATE
```

Replace `BASELINE`, `CANDIDATE`, and `BUDGET` with paths to schema-1 baseline
files.

`baseline` runs one warmup and three measured command invocations by default,
reads JSON benchmark output, compares with an existing baseline if present, and
writes the new baseline. Without `--baseline`, it writes a host-keyed file under
the repository's `benches/` directory. Use `--warmup`, `--runs`,
`--sample-count`, `--sample-size`, and `--variant` to adjust collection;
`--quiet` hides the comparison table, and `--print-path` prints the selected
path and exits. `--fast` runs once with no warmup, one sample, and one
iteration per sample.

`diff` prints a comparison between two baseline files. `check` treats the first
file as a set of absolute metric ceilings and requires both files to contain
exactly the same benchmark names.

Set `RUSTYBENCH_BENCH_RUSTFLAGS` to add Rust flags to the child benchmark
command launched by `baseline`; this does not change flags for the `rustybench`
tool itself.

## Syscall diagnostic

`rustybench syscalls` runs a Linux-only Docker and `strace` diagnostic. It needs
Docker and permission to use `SYS_PTRACE`. The trace parser bounds counts to
calls between `prctl(PR_SET_NAME, "BENCH_BEGIN")` and
`prctl(PR_SET_NAME, "BENCH_END")` markers. The runner emits these markers for
each synchronized sample batch in the diagnostic child process, including calls
from multiple worker threads. It defaults to 10 samples of 100 iterations. The
diagnostic is outside normal benchmark timing and can emit a human table or
schema-1 JSON with `--format human|json`.
