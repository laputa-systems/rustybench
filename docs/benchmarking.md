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
current thread. The `bench_values`, `bench_refs`, and `bench_loop_local`
families cover owned inputs, borrowed inputs, and repeated local loop work.
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
    // Use `size` as the input size.
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
