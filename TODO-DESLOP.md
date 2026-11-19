Prefix: DSLP
Next: 13

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier B). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

DSLP-1 base: `e01963bcbcb2db3c34a91baba0544dce25c5b4a0` (an
ancestor of the report revision). Ground and subsequent local work used macOS
ARM64 and pinned nightly 2026-11-09. PRs #18–#27 are merged; no PR is
parked. The remaining tickets below are P3, with no `Needs owner ·` items.

**Scorecard**

| Measure | Before | After / disposition |
| --- | --- | --- |
| Physical Rust lines in `src/` and `macros/` | 15,466 | 15,541; includes inline tests |
| Physical Rust lines in `tests/` and `benches/` | 286 | 310; excludes Python driver tests |
| Warm `make test` | 18.46 s at Ground | Not compared on a qualified host; #25's full cold build and test took 139.04 s under shared load |
| Coverage | Not measured | 83.57% lines, 75.28% branches (#25) |
| Bounded median mutation trial | 0 caught, 2 missed, 0 timeout, 0 unviable | 2 caught, 0 missed, 0 timeout, 0 unviable; about four runner-minutes in total. A separate selection produced 2 unviable and no test evidence. This is not a suite-wide gate. |
| Escape hatches | 89 unsafe, 14 allow, 21 non-test unwrap/expect, 157 casts | Same counts in Types (#24); the later dead-code step removed one allow. |
| Dead code | One unneeded allow | One attribute line removed; zero code lines and dependencies removed (#26). |
| Findings | Diagnose filed DSLP-6 and DSLP-9 (P2, fixed), and DSLP-7, DSLP-8 and DSLP-10 (P3). Bench filed DSLP-11 and DSLP-12 (P3). | Two defects fixed (#20 and #22); five P3 tickets open. No P1 or P2 remains open. |
| Performance | No qualified gate | Perf gate deferred: host busy. #27 supplies an entrypoint, not a performance verdict. |
| Escaped regressions | Not measured | None demonstrated by the cumulative completion review; this is not proof that none exist. |

The test work protects median ordering, normalization and extrema, human
report sample and iteration columns, budget verdicts, and advisory diff
output. Fault injections failed the appropriate tests; the two median
survivors represent one protected calculation, not two defects fixed.
Details and mutation configuration are in [the test audit](docs/deslop-tests.md).

**Steps and cost**

- **Ground (#18):** mapped contracts and 17 development consumers. The
  empty-target test took 191.75 s, warm test 18.46 s, and lint 123.13 s under
  a two-core guard and shared load. Hosted default test and lint checks passed;
  their CI minutes were not recorded.
- **Diagnose (#19–#23):** duplicate comparison names changed a displayed
  result with row order (#19); #20 rejects duplicates before output or baseline
  replacement. #21 reproduced malformed UTF-8, missing current directory and
  option-order failures; #22 fixed option expression parsing with a regression
  test and checked five downstream consumers. #23 found the empty JSON
  selection case. Wall time and CI minutes per slice were not recorded.
- **Types (#24):** nine candidate strict lints found no qualifying type fix;
  none adopted. The existing profile passed, and the lexical escape-hatch
  inventory did not increase. Wall time and CI minutes were not recorded.
- **Tests (#25):** added boundary assertions and three fault proofs; the
  bounded mutation trial cost about four runner-minutes. Coverage and the
  cold suite cost are above. CI minutes were not recorded.
- **Dead code (#26):** rustc and cargo-machete found no dead code or unused
  dependencies; removed one unnecessary suppression. Linux-only paths were
  inspected, not executed. Wall time and CI minutes were not recorded.
- **Bench (#27):** added tiered entrypoints, proven at Edit and one Change
  workload. Existing synthetic workloads lack a correctness oracle and do not
  isolate harness, allocation profiler or collector costs; DSLP-11 and
  DSLP-12 retain these gaps. Scenario and Full were not qualified or run.
  Wall time and CI minutes were not recorded.
- **Completion review:** reviewed the cumulative diff and changed callers;
  no new supported finding was established. Review wall time and CI minutes
  were not separately measured.

**Method feedback**

Exercise both orderings and both file roles at public boundaries: this would
have caught the option delimiter and duplicate-comparison failures earlier.
The initial mutation selection produced only unviable operators and no test
signal; a bounded median selection supplied useful evidence instead. Types
found no candidate worth adopting, and the dead-code scan found no code or
dependency to remove. Coverage alone does not establish assertion quality.

**Shared-file gaps**

`cargo-mutants` and `cargo-llvm-cov` are not on the crate mirror. The shared
registry needs to serve them before a retained mirror-backed mutation lane can
be qualified; the local coverage and bounded mutation runs do not qualify one.
A host-qualified warm-suite comparison and performance gate remain unmeasured.

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
