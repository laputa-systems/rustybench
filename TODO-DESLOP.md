Prefix: DSLP
Next: 13

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier B). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

Bench audit (DSLP-11 and DSLP-12, P3 missing evidence): the ordinary
`benches/bench.rs` sort, parse, event aggregation and histogram workloads
exercise runner input, counter and thread paths on synthetic data. They
black-box results without a correctness oracle; the collector checks report
inventory and sample counts. The ignored empty, fixed-work, size and threaded
controls expose calibrated operation timing, not total per-sample overhead.
No benchmark installs the allocation profiler, so its incremental cost and
allocation growth are unmeasured. The baseline CLI is exercised by collection,
but startup, collection, diff and check costs are not isolated. These missing
measurements could hide regressions; no regression was demonstrated.

The three macOS ARM64 files are historical observations: before overhead work
and current-thread ordinary workloads, and after-thread-fast-path controls.
They use one warmup, three rounds, ten samples and one iteration, with different inventories;
they are not interleaved, host-qualified gates or budgets. Median timing is
per operation after calibration; allocation traffic is not retained heap.
Raw process CPU and RSS/PSS snapshots cover the benchmark process, not its
process tree, and baseline aggregation drops those fields. No tail latency,
whole-tree memory, I/O or sustained throughput gate exists. Use Instruments or
platform CPU sampling for attribution, allocator profiling for heap traffic,
and the separate Linux `rustybench syscalls` diagnostic for syscall counts;
profiling changes timing and must be kept separate from timed runs. Operation
boundaries exist in the runner, but no cross-process trace is collected.

The tiered driver checks tools and checkout before running;
Cargo builds fresh artifacts. It records exclusions, commands, commit, dirty
state and per-workload logs, but does not reserve the host, screen contention,
verify cleanup of unrelated processes or qualify timings. No checked-in
budget exists; `diff` is advisory and `check` accepts explicit ceilings.
See [benchmarking](docs/benchmarking.md#rustybenchs-own-performance-workloads)
for selection, prerequisites and limitations. Scenario and Full remain
unqualified and Full was not run. Smallest next action: establish matched
profiler and collector workloads with an oracle and interleaved host-qualified
reference pairs before proposing budgets (DSLP-11, DSLP-12).

Dead code (DSLP-5): on macOS ARM64 with nightly 2026-11-09, the
workspace all-target, all-feature rustc scan found no dead-code warnings.
The `Action` suppression was unnecessary: all four predicates are used by
the runner or benchmark implementation. The test-only allocator, allocation
map constructor, split accessor, pool count and time-scale list serve the
allocation, split-vector, thread-pool and duration tests respectively.
Test modules are exercised by the test harness. No code or dependency was
removed; cargo-machete 0.10.1 found no unused dependencies. The scan also
reports 146 unreachable-public visibility diagnostics in private
implementation modules and test fixtures; these are visibility
diagnostics, not dead-code findings. Linux-only branches were inspected but
not compiled or run in this pass. The structural debloat review found no
proven behavior-preserving cut; documented public APIs, registrations,
runner options, reports and their tests remain contracts. Consumer tests and
performance replay were unnecessary for the suppression-only source change.

Tests (DSLP-4): the critical contracts map to existing tests, recorded in
[docs/deslop-tests.md](docs/deslop-tests.md). The bounded cargo-mutants
median trial (about four runner-minutes) found two survivors in median
selection; the strengthened allocation-median test catches both. New collector
cases check the human report columns and the `check` budget verdict and `diff`
percentage; injected faults in the budget comparison, median selection and
sample column each fail them. `make coverage` reports 83.57% lines and 75.28%
branches. A retained mutation lane is required only where the tools are
mirrored; cargo-mutants is not on the crate mirror.

Ground at `e01963bcbcb2db3c34a91baba0544dce25c5b4a0` used macOS ARM64
and pinned nightly 2026-11-09. The scoped fleet rule accepted
the existing Make test entry. Both hosted default checks passed.
[Default tests](https://github.com/laputa-systems/rustybench/actions/runs/44481013485) ·
[Default lint](https://github.com/laputa-systems/rustybench/actions/runs/44481013390)

Under a two-core CPU guard with two Cargo build jobs, repo-local
`TMPDIR` and an isolated `CARGO_TARGET_DIR`, the empty-target and
warm `make test` calls took 191.75 s and 18.46 s. Each ran 101
library cases, 30 binary cases, the collector acceptance and 84
doctests. Five cases were ignored. `make lint` passed in 123.13 s.
The observations include shared host load and compiler caching.

The inventory counted 15,466 physical Rust lines under `src/`
and `macros/` and 286 under `tests/` and `benches/`. The
fleet lists direct development use in defuddle-rs, diff-pretty,
e, fastermod, fz, gluon, hi-lite, img, ish, laputa, loc, lz,
rage, smolith, stuff, tm and xsh. The owning boundaries and
representative workload are in [Verification](AGENTS.md#verification).
Downstream execution and a readiness-qualified performance gate
remain open.

The CLI Diagnose slice built the locked `e8aa303` binary and checked
baseline collection, decoding and comparison. A schema-1 fixture with
two `fixture` rows at 10 and 30 ns and a candidate at 20 ns demonstrated
DSLP-6. Reversing the duplicate rows made `diff` change from `-33.33%`
to `+100.00%`. `check` rejected the same duplicate inventory. These
values are fabricated inputs to test presentation. README examples and
the development toolchain documentation now match the checked pin.
The library and macro review is recorded below.

The DSLP-6 repair uses the existing unique-name index for both
comparison inputs. With the built candidate, `diff` rejected duplicate
baseline and candidate rows before stdout. Both normal and quiet
collection preserved the previous file byte for byte on rejection.
Added and removed names still appeared in a valid comparison. The
CLI suite passed 32 tests with one existing ignore.

Collector replay on locked `6f5e939` used
`rustybench baseline --quiet --fast --root "$scratch" --baseline "$scratch/base.json" -- /bin/sh "$scratch/producer.sh"`.
The producer used `printf` to emit this report, with a name of
`café` for the first run and the byte `\377` for the second.

```json
{"schema":1,"benchmarks":[{"name":"café","median_ns":1,"alloc_count":0,"alloc_bytes":0,"max_alloc_count":0,"max_alloc_bytes":0,"sample_count":1,"iter_count":1}]}
```

Both runs exited zero. The second replaced the file with a row named
U+FFFD. A separate shell entered and removed its own empty scratch
directory, then ran the absolute binary with `baseline --print-path`.
It exited 101 from a current-directory panic. DSLP-7 and DSLP-8
retain these cases.

A path-dependency fixture on the same revision compiled a benchmark
attribute with `sample_count = 1` before
`name = if LOWER < UPPER { "lower" } else { "upper" }`, using
constant `u8` values 1 and 2. Moving the same `name` expression
before `sample_count` made `cargo check` fail with E0560. The two
fixtures are the control and reproduction for DSLP-9. The library and macro review is recorded below.


The DSLP-9 candidate uses the existing Rust parser for expression
and type boundaries and retains the original tokens for macro
generation. The targeted registration test passed for both
option orders and for a benchmark group. Reproduce with
`cargo test --offline --locked --lib comparisons_and_following_options_register_independently`.
At `bc6748c`, `make test` (84 library tests and doctests) and `make lint` passed on macOS ARM64. Reverting only `macros/src/attr_options.rs` and the manifest reproduces `expected :, found =` for both option orders and the group. With the requirement raised to 0.6 and both crates patched to the candidate, `cargo check --benches --workspace` passed for fastermod, loc, e, ish and gluon, identically to current main; the logs show the candidate macro crate loaded.

Diagnose reviewed library registration and filtering, benchmark sampling,
counters, allocation, reporting, timing and worker lifecycles, and macro
option parsing and generic expansion. A built runner reproduced DSLP-10:
a nonmatching JSON selection exited 0 without a report, while a matching
selection emitted schema 1. The earlier collector findings remain DSLP-7
and DSLP-8; the option-order failure was repaired in #22. Linux runtime,
downstream consumers, exhaustive unsafe soundness and performance gates
were not exercised in this review.

The DSLP-3 Types pass used macOS ARM64 and pinned nightly 2026-11-09,
base `15c7d9a52749035e45403b4facbd38f5efffc532`. Each candidate was
measured separately with `cargo clippy --workspace --all-targets
--all-features --locked -- -W <lint>`; all exited 0. Counts below are
unique emitted diagnostics, excluding Cargo summary lines and duplicate
builds. Tests and benches are included where compiled. Linux-only code was
not linted on this host.

| Candidate | Findings | Decision and reason |
| --- | ---: | --- |
| `unsafe_code` | 112 (102 library, 10 test-only) | Reject: registration, allocation and platform interfaces use unsafe; forbid would reject necessary code. |
| `clippy::wildcard_enum_match_arm` | 14 | Reject: fallback handling includes `Ordering`, token classification and CLI arguments; no demonstrated defect. |
| `clippy::match_wildcard_for_single_variants` | 1 | Reject: existing fallback in `src/entry/mod.rs`; no demonstrated defect. |
| `clippy::cast_possible_truncation` | 14 (8 library, 5 test-only, 1 bench) | Reject: bounded averages, saturated report conversion and deliberate benchmark truncation produce false positives; sample-index limits remain a lead, not a demonstrated defect. |
| `clippy::cast_sign_loss` | 4 | Reject: signed allocation maxima start at zero and remain nonnegative; report conversion is guarded and absolute difference deliberately preserves the unsigned magnitude. |
| `clippy::unwrap_used` | 174 (14 production, 160 tests/bench) | Reject: macro parser invariants, thread and sampling assumptions yield false positives; no qualifying defect was established. |
| `clippy::expect_used` | 6 (5 production, 1 test) | Reject: serialization and thread invariants yield false positives; the current-directory errors are already tracked as DSLP-8. |
| `clippy::redundant_clone` | 1 (test) | Reject: no production escape hatch or type defect. |
| `clippy::needless_pass_by_value` | 5 | Reject: ownership choices are not type defects or escape hatches. |

False-positive classification is conservative: unsafe has at least four
necessary sites; wildcard enum has at least four intentional fallbacks;
truncation has at least four bounded or deliberate conversions; sign loss has
four guarded or intentional conversions. The single-variant wildcard has one
intentional fallback. `unwrap_used` has at least four invariant-based sites;
`expect_used` has at least three invariant or serialization sites and two
sites already tracked as DSLP-8. The remaining diagnostics were not all
classified individually. The clone and pass-by-value diagnostics identify
ownership choices rather than typing defects (one and five respectively).
There are 15,821 tracked Rust lines; at least four false positives exceed the
one-per-5,000-line limit. No other candidate demonstrated a qualifying fix.

No new strict setting was adopted; the existing `make lint` profile passed.
Source was unchanged, so no contract was fixed and no new witness is claimed.
The producer and callers of report parsing, command selection, argument type
erasure and generic macro generation were inspected. Ranked remaining leads:
(1) sample counts and indices use `usize` to `u32` conversions; establishing
an executable overflow case requires a separate runtime and resource analysis;
(2) macro generic names and types are correlated optional fields after syntax
parsing and validated option selection; a type rewrite needs a negative witness
and must preserve accepted syntax; (3) the allocation initializer's unwrap
follows initialization and could be replaced without changing its contract.
DSLP-7, DSLP-8 and DSLP-10 remain open.

`python3 tools/deslop-type-count.py` counted before and after: 89 unsafe
blocks, 14 allow attributes, 21 non-test unwrap/expect calls and 157 `as`
cast-like expressions. This lexical inventory strips comments and string
literals, counts all first-party Rust targets including conditional platform
code, excludes test modules only for unwrap/expect, and excludes imports from
`as` counts. Generated Rust inside strings is outside the inventory. No hatch
count increased. The unchanged runtime and public contracts need no consumer
or performance replay for this documentary pass.

## Tickets








### DSLP-7 · P3 · Reject malformed UTF-8 in benchmark reports

- **Area:** deslop · collector
- **Where:** src/bin/rustybench.rs, run_benchmark; docs/commands.md, baseline collection
- **Problem:** On locked 6f5e939, baseline --quiet --fast accepted a zero-exit producer that emitted an otherwise valid schema-1 report with a raw 0xff in its name. It replaced a previous café baseline with a row named U+FFFD and exited 0. Lossy stdout decoding silently changes the benchmark identity before validation.
- **Recommendation:** Validate encoding on report lines before JSON decoding. Exercise a valid non-ASCII identity and a malformed child report and verify the second run preserves the prior baseline byte for byte.
- **Fix with:** error-model
- **Effort:** S

### DSLP-8 · P3 · Return a diagnostic when the current directory disappears

- **Area:** deslop · CLI
- **Where:** src/bin/rustybench.rs, baseline and syscalls root resolution
- **Problem:** On locked 6f5e939, a shell entered its own empty scratch directory, removed it with rmdir, then execed the absolute rustybench binary with baseline --print-path. The command panicked and exited 101. The syscalls command uses the same unchecked current directory lookup.
- **Recommendation:** Report the current-directory error through the existing CLI result path for commands with no explicit root. Replay the removed-directory case in an isolated subprocess.
- **Fix with:** error-model
- **Effort:** S

### DSLP-10 · P3 · Emit a JSON report for an empty selection

- **Area:** deslop · runner
- **Where:** src/rustybench.rs:114; docs/reports.md:5
- **Problem:** On 6bc9d297, the built collector fixture run with RUSTYBENCH_FIXTURE_CHILD=1 and --bench --format json definitely_no_matching_benchmark exits 0 with zero stdout bytes. With --exact collector::stable and the same JSON format it emits a schema-1 object. The early return for an empty filtered tree bypasses report serialization, so a JSON consumer cannot decode a successful empty selection.
- **Recommendation:** Emit a schema-1 object with an empty benchmarks array for an empty timed selection; exercise both empty and matching selections through the built runner.
- **Fix with:** error-model
- **Effort:** S

### DSLP-11 · P3 · Measure harness and allocation profiler overhead

- **Area:** deslop · benchmarks
- **Where:** benches/bench.rs; docs/timing.md
- **Problem:** The ignored controls time operations after runner calibration, not total per-sample harness cost, and the target does not install AllocProfiler. No matched profiler-on/off workload measures its cost or allocation growth; the historical macOS baselines do not supply this evidence.
- **Recommendation:** Add a checked representative consumer and matched profiler-on/off and per-sample controls; define completed work, scope and host-qualified interleaved comparisons before setting a budget.
- **Fix with:** refactor-benchmarks
- **Effort:** M


### DSLP-12 · P3 · Qualify collector and baseline performance evidence

- **Area:** deslop · benchmarks
- **Where:** benches/bench.rs; src/bin/rustybench.rs; docs/reports.md
- **Problem:** Baseline collection records wall time but the existing target does not isolate CLI startup, collection, comparison or process-tree resource cost. Three historical macOS files use different inventories and are not interleaved or qualified as budgets; raw process resources are not retained in baselines.
- **Recommendation:** Add a checked realistic report corpus and separate collector, diff and check workloads; measure process-tree resources and interleaved reference pairs with a host gate and attribution tools before proposing limits.
- **Fix with:** refactor-benchmarks
- **Effort:** M
