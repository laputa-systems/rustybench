# Benchmarking

Rustybench registers benchmarks with `#[rustybench::bench]` and groups with
`#[rustybench::bench_group]`. A function's module path becomes part of its
benchmark name; `name` changes the leaf name. See the [attribute reference](../src/lib.rs)
for all options and expansion examples.

## Inputs and ownership

Use a direct benchmark function for a single operation. Take a `Bencher` when
setup, generated inputs, or ownership must be explicit:

```rust
#[rustybench::bench]
fn sort(bencher: rustybench::Bencher) {
    bencher
        .with_inputs(|| vec![3, 1, 2])
        .bench_values(|mut values| values.sort_unstable());
}
```

Input generation with `with_inputs` happens outside the measured operation.
`bench_values` passes owned inputs to each iteration; use `bench_refs` to
inspect or mutate inputs by reference. Benchmark outputs are retained until
sample cleanup, so their destruction is not included in operation timing. Add a
benchmark whose operation explicitly drops its value when destruction is the
work being measured.

`bench` runs work across the configured threads; `bench_local` runs on the
current thread. Use `bench_values` or `bench_refs` for generated inputs, and
`bench_local_values` or `bench_local_refs` for their current-thread variants.
See the [`Bencher` API](../src/benchmark/mod.rs) for the full method reference.
Use `black_box` on benchmark inputs; returned values are black-boxed by the
runner, and `black_box_drop` makes intentional discards explicit.

## Variants and counters

Use `args` for runtime arguments and `types` or `consts` for generic expansion.
These options belong on a benchmark function, not its group. Repeatable
variants can come from a constant slice:

```rust
const SIZES: &[usize] = &[8, 64, 1024];

#[rustybench::bench(args = SIZES)]
fn process(size: usize) {
    rustybench::black_box(vec![0u8; size]);
}
```

Counters describe work done by one iteration. Use `counter` or `counters` for
fixed work and `input_counter` when the count depends on a generated input.
Convenience counters include bytes, characters, cycles, and items. See the
[counter API](../src/counter/mod.rs).

## Allocation profiling

Install `AllocProfiler::system()` as the global allocator to profile allocations
made by the benchmark executable, or pass it to an allocator-aware collection
for a local comparison. Profiling adds work to the measured path, so keep
allocator and profiler settings the same when comparing runs. Allocations from
unrelated threads are not a portable part of the results.

Read [timing](timing.md) for sampling, calibration, and deferred-drop details.

## Rustybench's own performance workloads

From the repository root, `make bench-list` lists the workloads and commands;
`make bench-check BENCH_ARGS='--tier Change --name runner'` checks prerequisites
without running them. `make bench` defaults to Edit. For example:

```sh
make bench BENCH_ARGS='--tier Change --name runner --dry-run'
make bench BENCH_ARGS='--tier Change --name runner'
make bench BENCH_ARGS='--tier Scenario --area runner --skip runner'
```

`--tier Edit|Change|Scenario|Full` sets a ceiling; Full is manual-only and
requires an explicit `--tier Full`. Repeat `--name`, `--area`, or `--skip` to
select workload names, areas, or excluded workload names. Name and area
filters intersect. Unknown selectors and empty selections fail. A dry run
prints commands and exclusions without preflight or writes. Preflight checks
Cargo, rustc and Git versions, platform, required checkout files,
and free disk before any workload starts. Cargo rebuilds stale artifacts; no
external fixtures or services are used. Host load and isolation are not checked.
Python 3, Git, the pinned Rust toolchain and at least 1 GiB free are required.
Rustup toolchain overrides must be cleared to use the checkout pin.

| Name | Tier / area | Purpose | Estimated cost, warm build |
| --- | --- | --- | --- |
| `accounting` | Edit / accounting | Allocation median correctness test | seconds |
| `runner` | Change / runner | Collect one sort benchmark through the CLI | minutes |
| `overhead` | Scenario / runner | Collect ignored harness controls, three measured rounds | minutes |
| `suite` | Full / runner | Collect all ordinary representative workloads, three measured rounds | minutes |

Runs create `.work/bench/<date>/summary.txt` and a directory per workload
with `run.log` and, for collection, `baseline.json`. The summary records
exclusions, exit codes, commands, duration, commit and dirty state. A failed
workload fails the run. Timing is unqualified: samples are not screened for
host contention, and completion does not establish a performance verdict.
The collector verifies report inventories and nonzero samples, but the
representative workload results have no correctness oracle. For comparisons,
use `cargo run --bin rustybench -- diff BASELINE CANDIDATE` or `check BUDGET
CANDIDATE`; choose matching workloads and qualified hosts. `check` applies
absolute ceilings, and historical baseline observations are not budgets.
The checked-in macOS baselines are preserved; they have different inventories
and should not be treated as interchangeable references.
