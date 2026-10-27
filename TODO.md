Prefix: RUST
Next: 25

### RUST-24 · P3 · `allocator_api` warns on nightlies after the pin

- **Area:** Build
- **Where:** `src/lib.rs:4`, `README.md:7`, `rust-toolchain.toml:2`
- **Problem:** The crate enables `allocator_api`. README tells users to install the latest nightly. On nightly 1.101.0 (16d1ffac1 2026-10-26) that feature warns as partially stabilized and points at `allocator_ext`. The pinned toolchain `nightly-2026-09-15` still builds cleanly. A warning-free build does not match the README's "latest nightly" instruction, and the feature can disappear.
- **Recommendation:** Move the allocator API onto the stabilized surface, or onto `allocator_ext` for what is still unstable, and say which nightly README actually supports.
- **Fix with:** none
- **Effort:** S
