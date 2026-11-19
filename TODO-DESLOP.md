Prefix: DSLP
Next: 7

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


### DSLP-6 · P2 · Reject duplicate names before displaying baseline comparisons

- **Area:** Baseline comparison
- **Where:** src/bin/rustybench.rs:497-518, 615-643, 697-723
- **Problem:** The built diff command accepted two records named fixture with median_ns values 10 and 30, then displayed either a 33.33 percent decrease or a 100 percent increase against a candidate at 20, depending only on row order. The same input was rejected by check. See Report for the fixture.
- **Recommendation:** Validate the unique-name invariant before displaying a diff and before comparing with a previous baseline. Reuse the existing duplicate diagnostic path. Test duplicate names in both input roles, preserve missing and added benchmark comparisons, and verify malformed existing baselines survive a failed collection.
- **Fix with:** durable-state
- **Effort:** M

