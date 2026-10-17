Prefix: RUST
Next: 17

### RUST-10 · P2 · Compute allocation medians from allocation samples

- **Area:** Data
- **Where:** `src/benchmark/mod.rs:1226-1228,1353-1376`
- **Problem:** The samples in `median_samples` are selected by elapsed time. The allocation report averages allocation tallies from those same time-selected samples, so its median can differ from the median allocation count and size when allocation behavior varies independently of runtime.
- **Recommendation:** Select and summarize allocation tallies by their own values while retaining the sample-to-allocation association.
- **Fix with:** `none`
- **Effort:** S (hours)

### RUST-11 · P3 · Honor nested ignore overrides in terse listing

- **Area:** Config
- **Where:** `src/rustybench.rs:167-200,484-489`
- **Problem:** Terse listing checks a parent group's `ignore` value and skips that subtree before visiting children. Unlike normal traversal, it does not merge child options over parent options, so a nested `ignore = false` benchmark is omitted under an ignored group. A probe under `NEXTEST=1` confirmed the child is missing from `--list --format terse` output.
- **Recommendation:** Apply the same parent-to-child option override rules during terse listing as during normal traversal.
- **Fix with:** `reference-parity`
- **Effort:** S (hours)

### RUST-12 · P3 · Capture process counters before releasing sample workers

- **Area:** Data
- **Where:** `src/benchmark/mod.rs:965-972,1015-1018`
- **Problem:** With syscall markers disabled, the process-resource start snapshot is taken after the second start-barrier wait releases workers. A worker can begin its timed sample before the snapshot, so process-wide CPU and fault deltas omit part of the work for multithreaded samples.
- **Recommendation:** Take the start snapshot at the shared boundary before releasing workers, regardless of syscall-marker configuration.
- **Fix with:** `isolate-effects`
- **Effort:** M (days)

### RUST-13 · P3 · Verify LLVM archives before extracting them in Docker

- **Area:** Security
- **Where:** `Dockerfile:23-32`
- **Problem:** The Docker build downloads versioned LLVM archives over HTTPS and extracts them as the compiler toolchain without checking a published digest or signature. A replaced upstream artifact becomes executable build input.
- **Recommendation:** Verify the upstream release digest or signature before extraction.
- **Fix with:** `none`
- **Effort:** S (hours)

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
