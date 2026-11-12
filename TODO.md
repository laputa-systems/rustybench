Prefix: RUST
Next: 35

### RUST-24 · P3 · `allocator_api` warns on nightlies after the pin

- **Area:** Build
- **Where:** `src/lib.rs:4`, `rust-toolchain.toml:2`
- **Problem:** The crate still enables `allocator_api`. The earlier review
  recorded a partial-stabilization warning pointing to `allocator_ext` on
  nightly 1.101.0 (16d1ffac1 2026-10-26). That newer-toolchain observation was not
  independently rerun in this review. The pinned `nightly-2026-09-15` build and
  tests pass, and README now names that tested toolchain explicitly.
- **Recommendation:** Reproduce the warning on the intended next toolchain,
  then migrate to the stabilized allocator surface or `allocator_ext` as
  required. Keep the current pin while investigating, and validate any pin
  update against the workspace build and tests.
- **Fix with:** none
- **Effort:** S (hours)
