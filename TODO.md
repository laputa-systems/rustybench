Prefix: RUST
Next: 40
### RUST-35 · P3 · Reject malformed UTF-8 in benchmark reports

- **Area:** collector
- **Where:** src/bin/rustybench.rs, run_benchmark; docs/commands.md, baseline collection
- **Problem:** On locked 6f5e939, baseline --quiet --fast accepted a zero-exit producer that emitted an otherwise valid schema-1 report with a raw 0xff in its name. It replaced a previous café baseline with a row named U+FFFD and exited 0. Lossy stdout decoding silently changes the benchmark identity before validation.
- **Recommendation:** Validate encoding on report lines before JSON decoding. Exercise a valid non-ASCII identity and a malformed child report and verify the second run preserves the prior baseline byte for byte.
- **Fix with:** error-model
- **Effort:** S

### RUST-36 · P3 · Return a diagnostic when the current directory disappears

- **Area:** CLI
- **Where:** src/bin/rustybench.rs, baseline and syscalls root resolution
- **Problem:** On locked 6f5e939, a shell entered its own empty scratch directory, removed it with rmdir, then execed the absolute rustybench binary with baseline --print-path. The command panicked and exited 101. The syscalls command uses the same unchecked current directory lookup.
- **Recommendation:** Report the current-directory error through the existing CLI result path for commands with no explicit root. Replay the removed-directory case in an isolated subprocess.
- **Fix with:** error-model
- **Effort:** S

### RUST-37 · P3 · Emit a JSON report for an empty selection

- **Area:** runner
- **Where:** src/rustybench.rs:114; docs/reports.md:5
- **Problem:** On 6bc9d297, the built collector fixture run with RUSTYBENCH_FIXTURE_CHILD=1 and --bench --format json definitely_no_matching_benchmark exits 0 with zero stdout bytes. With --exact collector::stable and the same JSON format it emits a schema-1 object. The early return for an empty filtered tree bypasses report serialization, so a JSON consumer cannot decode a successful empty selection.
- **Recommendation:** Emit a schema-1 object with an empty benchmarks array for an empty timed selection; exercise both empty and matching selections through the built runner.
- **Fix with:** error-model
- **Effort:** S

### RUST-38 · P3 · Measure harness and allocation profiler overhead

- **Area:** benchmarks
- **Where:** benches/bench.rs; docs/timing.md
- **Problem:** The ignored controls time operations after runner calibration, not total per-sample harness cost, and the target does not install AllocProfiler. No matched profiler-on/off workload measures its cost or allocation growth; the historical macOS baselines do not supply this evidence.
- **Recommendation:** Add a checked representative consumer and matched profiler-on/off and per-sample controls; define completed work, scope and host-qualified interleaved comparisons before setting a budget.
- **Fix with:** refactor-benchmarks
- **Effort:** M

### RUST-39 · P3 · Qualify collector and baseline performance evidence

- **Area:** benchmarks
- **Where:** benches/bench.rs; src/bin/rustybench.rs; docs/reports.md
- **Problem:** Baseline collection records wall time but the existing target does not isolate CLI startup, collection, comparison or process-tree resource cost. Three historical macOS files use different inventories and are not interleaved or qualified as budgets; raw process resources are not retained in baselines.
- **Recommendation:** Add a checked realistic report corpus and separate collector, diff and check workloads; measure process-tree resources and interleaved reference pairs with a host gate and attribution tools before proposing limits.
- **Fix with:** refactor-benchmarks
- **Effort:** M

