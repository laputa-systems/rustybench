# Rustybench working guide

Rustybench is a small benchmark runner. It keeps the useful registration and Bencher
ideas, but its command line parser is lexopt, its
machine-readable format is miniserde-based JSON, and its benchmark tooling is built
into the crate instead of depending on a large external CLI stack.

The supported target contract is macOS and Linux. Windows, WebAssembly, and other
targets are intentionally out of scope unless that contract is changed explicitly.

The crate namespace and public runner type are Rustybench. The procedural macros
default to the rustybench crate path. A consumer that renames the dependency
must pass crate = ... to the benchmark attribute.

The benchmark semantics are described below: nested benchmark names, generic and
const expansion, runtime arguments, input ownership, throughput counters,
allocation profiling, thread scaling, deferred drops, and sample scaling. Windows,
WebAssembly, and other targets are outside the current contract.

## Using rustybench

A consumer adds a local path dependency and disables Cargo's default benchmark
harness:

    [dev-dependencies]
    rustybench = { path = "../rustybench" }

    [[bench]]
    name = "example"
    harness = false

A benchmark executable calls rustybench::main:

    use rustybench::black_box;

    fn main() {
        rustybench::main();
    }

    #[rustybench::bench]
    fn add() -> i32 {
        black_box(1) + black_box(42)
    }

Run a benchmark with Cargo:

    cargo bench --bench example
    cargo bench --bench example -- add
    cargo bench --bench example -- --list

Cargo's benchmark harness arguments --bench, --nocapture, and --show-output are
accepted for compatibility. A benchmark executable can also receive --test or
--bench explicitly after the Cargo separator. Test mode runs each selected benchmark
once without timing it. cargo test --benches and cargo bench -- --test are useful
compile and smoke checks.

## Registration and benchmark names

rustybench::bench can be placed on functions, associated functions, and methods in
nested modules. The module path becomes part of the benchmark name. The
rustybench::bench_group attribute applies options to a module or nested group.

The benchmark attribute supports:

* name for the leaf name;
* args for runtime argument expansion;
* types for generic type expansion;
* consts for const-generic expansion;
* counter and counters for throughput measurements;
* bytes_count, chars_count, cycles_count, and items_count convenience counters;
* sample_count, sample_size, min_time, max_time, threads, and skip_ext_time;
* ignore for benchmarks excluded by default.

Group attributes can set timing, thread, counter, and other group-wide options, but
args, types, and consts belong on the benchmark they expand.

The public API examples in src/lib.rs and the macro reference cover the supported
registration forms. Keep new usage examples close to the API definition they explain.
Useful patterns include an `AllocProfiler` global allocator for allocation metrics,
`bench_group(threads = THREADS)` for thread scaling, `with_inputs` for setup outside
the measured operation, `types` and `consts` for expansion, and `counter` or
`input_counter` for throughput metrics.

## Benchmark design patterns

Use a small constant slice for repeatable argument or thread variants, for example
`const SIZES: &[usize] = &[8, 64, 1024];` and
`#[rustybench::bench(args = SIZES)]`. A threaded group can use
`const THREADS: &[usize] = &[0, 1, 4, 16];`; zero means available parallelism.

Generate large or randomized inputs with `with_inputs` so setup stays outside the
measured operation. Use `bench_values` when the operation should consume and drop each
input, and `bench_refs` when it should mutate or inspect an input in place. Attach a
constant `counter` for fixed work per iteration, or an `input_counter` when the work
depends on the generated input. Use `black_box` for values and `black_box_drop` when
the result is intentionally discarded.

For allocation comparisons, install `AllocProfiler::system()` as the global allocator
in the benchmark executable. Keep allocator choice and profiling configuration the
same across compared baselines; profiling changes the measured path.

## Bencher and input ownership

Take a rustybench::Bencher parameter when setup, input generation, or ownership
needs to be explicit:

    #[rustybench::bench]
    fn sort(bencher: rustybench::Bencher) {
        bencher
            .with_inputs(|| vec![3, 1, 2])
            .bench_values(|mut values| values.sort_unstable());
    }

The main methods are bench for threaded closures, bench_local for a current-thread
closure, with_inputs for generated inputs, bench_values and bench_local_values for
owned inputs, bench_refs and bench_local_refs for borrowed inputs, input_counter for
input-dependent counters, count_inputs_as for item counts, and bench_loop_local for
repeated current-thread loop work.

Input generation is outside the measured operation. Outputs are retained until the
sample cleanup phase so destruction does not accidentally become part of the timing.
Use the values forms when that ownership behavior is wanted. Add an explicit
benchmark for destruction when destruction itself is the behavior under test.
black_box and black_box_drop are available for preventing optimization and making
drop behavior explicit.

## Timing, sampling, and allocation metrics

The default sample count is 100. If sample_size is not supplied, rustybench chooses
an iteration count from the timing target. sample_size sets iterations per sample;
threaded runs round the work to a suitable multiple. min_time and max_time bound
the tuning and collection work, while skip_ext_time excludes time spent outside
the measured closure when that distinction is available.

Rustybench uses the operating-system monotonic timer for all measurements. This keeps
timing behavior portable and avoids architecture-specific calibration or fallback
paths.

The built-in AllocProfiler is the default global allocator wrapper around the
system allocator. Reports include allocation count and allocated bytes, plus the
maximum live allocation count and bytes observed during a sample. Allocation
profiling changes the measured path; compare like-for-like runs. Allocations made
by unrelated threads are not a portable part of the benchmark contract.

The counters are BytesCount, CharsCount, CyclesCount, and ItemsCount. Byte rates
are decimal by default and can be displayed in binary units with
--bytes-format binary. A counter describes work represented by one iteration, so
the counter value must match the benchmark's actual input.

## Benchmark command line

The runner accepts positional regular-expression filters and these options:

    --test
    --list
    --skip FILTER
    --exact
    --ignored
    --include-ignored
    --sort kind|name|location
    --sortr kind|name|location
    --sample-count N
    --sample-size N
    --threads N[,N...]
    --min-time SECONDS
    --max-time SECONDS
    --skip-ext-time[=BOOL]
    --items-count N
    --bytes-count N
    --bytes-format decimal|binary
    --chars-count N
    --cycles-count N
    --format pretty|terse|json

Use --exact to make filters exact names. --skip excludes matching names. --ignored
runs only ignored benchmarks, while --include-ignored includes them with ordinary
benchmarks. A thread count of zero means the available parallelism. The
RUSTYBENCH_* environment variables are: RUSTYBENCH_SORT, RUSTYBENCH_SORTR,
RUSTYBENCH_SAMPLE_COUNT, RUSTYBENCH_SAMPLE_SIZE,
RUSTYBENCH_THREADS, RUSTYBENCH_MIN_TIME, RUSTYBENCH_MAX_TIME,
RUSTYBENCH_SKIP_EXT_TIME, RUSTYBENCH_ITEMS_COUNT, RUSTYBENCH_BYTES_COUNT,
RUSTYBENCH_BYTES_FORMAT, RUSTYBENCH_CHARS_COUNT, and RUSTYBENCH_CYCLES_COUNT.

## JSON output and tooling

--format json writes one report object to stdout. Schema 1 has a schema number and
a benchmarks array. Each benchmark record contains name, median_ns, alloc_count,
alloc_bytes, max_alloc_count, max_alloc_bytes, sample_count, and iter_count.
Keep this shape stable or increment the schema and update its consumers.

The rustybench binary also contains the replacement for the repository's benchmark
scripts:

    cargo run --manifest-path ../rustybench/Cargo.toml -- baseline --root . --baseline benches/baseline.json -- cargo bench --bench example
    cargo run --manifest-path ../rustybench/Cargo.toml -- diff benches/baseline.json benches/candidate.json

baseline runs one warmup and three measured command invocations by default, aggregates
the JSON benchmark records, compares with an existing file, and writes a host-keyed
JSON baseline. Use --warmup N, --runs N, --sample-count N, --sample-size N,
--variant NAME, --quiet, or --print-path as needed. --fast means no warmup, one
measured run, one sample, and one iteration per sample. Baseline files include host,
platform, architecture, mode, run counts, timing totals, per-run suite times, and
the aggregated benchmark metrics. Current schema-1 baseline records also include
optional per-run timing medians; older schema-1 files omit that field and remain
readable. Additive baseline fields must stay optional. A change that makes an
existing field mandatory or changes its meaning must increment the schema and add
a compatibility test. Baseline files are JSON only; tab-separated files are rejected.

RUSTYBENCH_BENCH_RUSTFLAGS applies extra Rust flags to the child benchmark command
launched by baseline. It does not alter the rustybench tool itself.

syscalls is a Linux-only Docker and strace diagnostic. It needs Docker, the syscall
Dockerfile, permission to use SYS_PTRACE, and benchmark output markers named
BENCH_BEGIN and BENCH_END. It is a diagnostic path, not part of normal benchmark
measurement.

## Source map and checks

The runner implementation is in src/rustybench.rs, command parsing is in src/cli.rs,
JSON report types are in src/report.rs, and the standalone baseline, diff, and
syscalls commands are in src/bin/rustybench.rs. Registration and expansion live in
macros/src. Keep the public Rustybench runner name stable after publication.

Useful checks after changing rustybench:

    cargo test --workspace
    cargo check --workspace --all-targets
    cargo bench --bench bench -- --list
    cargo bench --bench bench -- --format json --sample-count 1 --sample-size 1
    git diff --check

Do not run pre-commit hooks, create commits, or push remotes. Preserve unrelated
working-tree changes.
