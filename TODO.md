Prefix: RUST
Next: 17

### RUST-14 · P3 · Compare both benchmark names in numeric sorting

- **Area:** Config
- **Where:** `src/config/mod.rs:102-137`
- **Problem:** The integer comparison parses `a` for both tuple elements, including in the signed-integer fallback. Numeric arguments therefore compare equal instead of sorting by their values. Running the sample benchmark with `--sortr name` produced `100, 3, 8` instead of descending numeric order.
- **Recommendation:** Parse and compare `a` with `b`, and retain an unsorted numeric input in the comparator regression coverage.
- **Fix with:** `none`
- **Effort:** S (hours)

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
