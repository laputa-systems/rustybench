Prefix: RUST
Next: 17

### RUST-15 · P3 · Match Linux resource fields to the target C long width

- **Area:** Data
- **Where:** `src/resource.rs:391-410`
- **Problem:** Linux `RawTimeval` and `RawRusage` use a hard-coded `i64` for C `long`. On 32-bit Linux, libc's `c_long` is 32 bits, so `getrusage` writes a different structure layout and the parsed process counters are invalid.
- **Recommendation:** Use the target's C ABI `c_long` type for every `getrusage` field.
- **Fix with:** `os-boundaries`
- **Effort:** S (hours)

### RUST-16 · P3 · Avoid collisions in generated benchmark statics

- **Area:** Build
- **Where:** `macros/src/lib.rs:96-99,547-549`
- **Problem:** Generated static names uppercase the source function name. Distinct valid Rust identifiers such as `bench` and `BENCH` therefore produce the same generated static and fail compilation when used as benchmarks in one module.
- **Recommendation:** Generate collision-resistant identifiers independent of source spelling, such as using a unique per-item token rather than uppercased names.
- **Fix with:** `api-surface-and-module-boundaries`
- **Effort:** S (hours)
