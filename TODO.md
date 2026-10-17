Prefix: RUST
Next: 17

### RUST-16 · P3 · Avoid collisions in generated benchmark statics

- **Area:** Build
- **Where:** `macros/src/lib.rs:96-99,547-549`
- **Problem:** Generated static names uppercase the source function name. Distinct valid Rust identifiers such as `bench` and `BENCH` therefore produce the same generated static and fail compilation when used as benchmarks in one module.
- **Recommendation:** Generate collision-resistant identifiers independent of source spelling, such as using a unique per-item token rather than uppercased names.
- **Fix with:** `api-surface-and-module-boundaries`
- **Effort:** S (hours)
