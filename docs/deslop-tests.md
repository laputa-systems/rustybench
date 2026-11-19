# Tests audit

This audit used macOS ARM64, nightly 2026-11-09, base b64a43a6, and the
contracts and tiers in `AGENTS.md`. The full suite before changes passed 102
library and 32 tool tests, the collector, and 84 doctests. Four library and
one tool probes are ignored. It took 139.04 seconds including a cold build;
shared host load prevents treating this as a timing gate.

`make coverage` with cargo-llvm-cov 0.9.1 passed after the changes. Its
summary reports 83.57% lines and 75.28% branches, across the instrumented
files. No before-change coverage report was collected. Coverage is a report,
not a threshold or evidence of an effective assertion.

The collector now exercises equal and exceeded budgets through the binary,
checks that `diff` remains successful on a regression, and checks the human
report's columns, sample count, and iteration count. Temporarily changing
`value > limit` to `value >= limit` made `cargo test --locked --test collector`
exit 101 at the equal-budget check. Temporarily printing iteration count in
the samples column made the same command exit 101 with `["6", "6"]` instead
of `["2", "6"]`. Both production changes were reverted.

The statistics scenario checks sorted odd and even medians, extrema and
per-iteration normalization as well as allocation medians. Replacing the
middle slice with the first sample made the focused library test exit 101:
100 instead of 200 picoseconds. The production change was reverted.

Mutation used cargo-mutants 27.1.0, one job, `--in-place --baseline run
--timeout 30 --build-timeout 180`, the library test selector
`-- --lib alloc_median_follows_allocation_order_not_time_order`, and
`--file src/benchmark/mod.rs`. A first selection, `--re
'src/benchmark/mod.rs:1336:33:'`, took 42 seconds and produced two unviable
mutants: `FineDuration` does not implement the substituted operators. It
provides no test evidence. The median selection, `--re
'src/benchmark/mod.rs:1348:28:'`, took two minutes before the new assertions:
two missed, zero caught, unviable or timed out. Both survivors changed the
integer median calculation. With the assertions restored, the same selection
took 74 seconds: two caught, zero missed, unviable or timed out. The `%`
mutant failed on 0 versus 200; the `*` mutant failed on 1000 versus 250.
There were no survivors after the change. Total observed runner time was
about four minutes; this was an investigation of the median calculation,
not a package-wide mutation gate.

The diff selection recipe and CI mutation lane are not qualified by this
experiment. They remain work for DSLP-4; a missing gate or an unconfirmed
timeout must not be treated as a pass. The warm suite cost comparison also
requires a host that passes `perf-gate-ready` before and after measurement.
