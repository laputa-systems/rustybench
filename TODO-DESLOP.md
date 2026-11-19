Prefix: DSLP
Next: 10

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier B). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

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
The remaining library, macro and collector review stays in DSLP-2.

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
fixtures are the control and reproduction for DSLP-9. The remaining
library and macro review stays in DSLP-2.


The DSLP-9 candidate uses the existing Rust parser for expression
and type boundaries and retains the original tokens for macro
generation. The targeted registration test passed for both
option orders and for a benchmark group. Reproduce with
`cargo test --offline --locked --lib comparisons_and_following_options_register_independently`.
At `bc6748c`, `make test` (84 library tests and doctests) and `make lint` passed on macOS ARM64. Reverting only `macros/src/attr_options.rs` and the manifest reproduces `expected :, found =` for both option orders and the group. With the requirement raised to 0.6 and both crates patched to the candidate, `cargo check --benches --workspace` passed for fastermod, loc, e, ish and gluon, identically to current main; the logs show the candidate macro crate loaded.

## Tickets


### DSLP-2 · P2 · Diagnose: ultrareview into this backlog

- **Area:** deslop · diagnose
- **Where:** whole repository
- **Problem:** No review has ranked this repository's risks for the deslop pass.
- **Recommendation:** Run the deslop Diagnose step. Accept: the diff touches only docs, comments, and this file, and every finding names its Fix with.
- **Fix with:** deslop
- **Effort:** S

### DSLP-3 · P2 · Types: strict profile and typing findings

- **Area:** deslop · types
- **Where:** type checker and lint configuration; Rust sources
- **Problem:** The strict-profile candidates in the deslop skill's tools reference have not been measured here.
- **Recommendation:** Run the deslop Types step. Accept: clean at the adopted profile, escape hatches not increased, witnesses recorded.
- **Fix with:** deslop
- **Effort:** M

### DSLP-4 · P2 · Tests: coverage, boundary gaps, mutation

- **Area:** deslop · tests
- **Where:** test suites and CI
- **Problem:** Coverage and mutation results are unmeasured; no deslop mutation lane exists.
- **Recommendation:** Run the deslop Tests step. Accept: no undispositioned survivor in critical code, the test cost budget holds, and the lane is green where the tools are mirrored.
- **Fix with:** deslop
- **Effort:** M

### DSLP-5 · P2 · Dead code: tools and debloat

- **Area:** deslop · dead code
- **Where:** whole repository
- **Problem:** No dead-code scan has run.
- **Recommendation:** Run the deslop Dead code step. Accept: tools clean or allowlisted with reasons; walk-backs filed as Needs owner tickets.
- **Fix with:** deslop
- **Effort:** M



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
