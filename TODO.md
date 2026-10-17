Prefix: RUST
Next: 22

### RUST-17 · P3 · Reject a zero sample size

- **Area:** Config
- **Where:** `src/cli.rs:264-271`, `src/benchmark/options.rs:12`, `src/benchmark/mod.rs:1309-1310,1383`
- **Problem:** `--sample-count 0` is rejected, but `--sample-size 0` is accepted and the report path divides by the sample size. `cargo bench --bench bench -- --sample-size 0 --sample-count 1` panics with `attempt to divide by zero` at `src/benchmark/mod.rs:1383` instead of naming the flag. `BenchOptions::sample_size` and `Bencher::sample_size(0)` bypass the CLI check entirely.
- **Recommendation:** Reject a zero sample size where it is parsed and where options are resolved, with the same message style as the sample-count check.
- **Fix with:** `error-model`
- **Effort:** S (hours)

### RUST-18 · P3 · Verify the rustup downloads in the Docker build

- **Area:** Security
- **Where:** `Dockerfile:42-43`
- **Problem:** The two `rustup-init` binaries are fetched with `ADD` and no checksum from an unversioned URL, so a replaced upstream artifact becomes executable build input and the pinned toolchain version floats.
- **Recommendation:** Pin the rustup version in the URL and add `--checksum=sha256:` for both binaries, using the digests published for that version.
- **Fix with:** `os-boundaries`
- **Effort:** S (hours)

### RUST-19 · P4 · Compare prereleases in the release script's version plan

- **Area:** Tooling
- **Where:** `tools/bump_minor_release.sh:234-281`
- **Problem:** `version_plan`'s `compare_versions` strips prerelease and build metadata before comparing, so `1.4.0-rc.1` and `1.4.0` compare equal. `version_at_least` follows Cargo semver precedence; this second comparator still does not, so `current_vs_latest` can read 0 where semver says the local version is newer and the "local version is newer than published" guard diverges from the comparator the rest of the script uses.
- **Recommendation:** Have both call sites use one semver-precedence comparator instead of two that disagree.
- **Fix with:** `single-source-of-truth`
- **Effort:** S (hours)

### RUST-20 · P4 · Keep a top-level comparison from swallowing consts commas

- **Area:** Macros
- **Where:** `macros/src/tokens.rs:33-86`
- **Problem:** `split_top_level` opens angle-bracket depth on any `<` that is not part of a multi-char operator, so a comparison between two const expressions at the top level of a `consts` list is treated as a generic opener and every following comma is swallowed. `#[bench(consts = [1 < 3, 2 < 3])] fn f<const B: bool>()` registers one generic bench instead of two, silently dropping the second value.
- **Recommendation:** Open angle depth only where a generic argument list can start (after a path segment, `::`, or `>`), not after a literal, a closing delimiter, or an operator.
- **Fix with:** `make-invalid-states-unrepresentable`
- **Effort:** S (hours)

### RUST-21 · P4 · Split `consts` by reparsing instead of by token heuristics

- **Area:** Macros
- **Where:** `macros/src/tokens.rs:33-86`
- **Problem:** The angle-depth heuristic cannot tell a comparison between two identifiers from a generic argument list, so `#[bench(consts = [LEN < 3, 4])]` still swallows the comma after `3` and registers one generic bench instead of two, silently dropping the rest of the list.
- **Recommendation:** Decide the split by parsing: split at commas and keep fragments that parse as the expected type or expression, rather than by tracking angle depth at the token level.
- **Fix with:** `root-cause`
- **Effort:** M (days)
