Prefix: RUST
Next: 17

### RUST-4 · P2 · Parse commas inside generic benchmark types

- **Area:** Architecture
- **Where:** `macros/src/tokens.rs:33-47`, `macros/src/attr_options.rs:299`, `macros/src/lib.rs:363`
- **Problem:** `split_top_level` treats every comma punctuation as a separator and does not track angle-bracket nesting. A macro input such as `types = [HashMap<u8, u8>]` is split into invalid type fragments; compiling a probe with this input fails.
- **Recommendation:** Split only commas at the top-level delimiter nesting of the Rust type syntax.
- **Fix with:** `api-surface-and-module-boundaries`
- **Effort:** M (days)

### RUST-5 · P2 · Preserve generic type identity in benchmark paths

- **Area:** Navigability
- **Where:** `src/entry/generic.rs:45-59,83-94`, `src/entry/tree.rs:12-24,26-48`
- **Problem:** Generic type parents are keyed by `display_name`, which strips module components from type names. Distinct types with the same short name therefore share one tree parent and can produce ambiguous or collapsed benchmark paths.
- **Recommendation:** Keep type identity separate from its display label and disambiguate colliding paths.
- **Fix with:** `api-surface-and-module-boundaries`
- **Effort:** M (days)

### RUST-6 · P2 · Release parallel benchmark workers after a panic

- **Area:** Concurrency
- **Where:** `src/benchmark/mod.rs:953-977`, `src/util/thread/pool.rs:130-143,264-269`
- **Problem:** Benchmark threads synchronize on a barrier after running the sample. If one benchmark closure panics first, the thread pool catches the panic, but its peers can remain blocked at the barrier and the main thread waits forever for the task to finish.
- **Recommendation:** Make sample synchronization cancelable and propagate a worker panic to all participants before waiting for the pool.
- **Fix with:** `lifecycle-concurrency`
- **Effort:** M (days)

### RUST-7 · P2 · Clear input counters when setting a static counter

- **Area:** Types
- **Where:** `src/counter/collection.rs:43-56,64-80`, `src/benchmark/mod.rs:822-835,1234-1244`
- **Problem:** `set_input_counter` clears static values, but `set_counter` leaves an existing input counter closure installed. Chaining an input counter and then a static counter of the same kind keeps input-based indexing enabled; the static value shifts the input sample counts and produces incorrect report values.
- **Recommendation:** Make setting either counter source clear the other source for that counter kind.
- **Fix with:** `make-invalid-states-unrepresentable`
- **Effort:** S (hours)

### RUST-8 · P2 · Keep resource snapshots from suppressing worker allocation samples

- **Area:** Concurrency
- **Where:** `src/alloc.rs:14`, `src/util/sync.rs:39-54`, `src/resource.rs:135-139`, `src/benchmark/mod.rs:917-924,1062-1063,1117-1118`
- **Problem:** `IGNORE_ALLOC` is one shared atomic flag. While the main thread captures process resources, `capture_unprofiled` sets it for every thread. A worker saving its allocation sample during that interval sees the flag and returns without copying its thread-local tally, so multithreaded allocation reports can silently omit samples.
- **Recommendation:** Scope allocation suppression to the thread taking the resource snapshot, and synchronize allocation sample collection across workers.
- **Fix with:** `lifecycle-concurrency`
- **Effort:** M (days)

### RUST-9 · P2 · Bound sample-size tuning by batch setup cost

- **Area:** Performance
- **Where:** `src/benchmark/mod.rs:771-787,1081-1103`, `src/benchmark/defer.rs:64-85`
- **Problem:** Tuning doubles `sample_size` based on the timed closure duration, then reserves and initializes storage for the full batch before checking the time budget again. Benchmarks with large generated inputs can retain enough inputs in one tuning batch to exhaust memory, and expensive setup or drops can make a batch run far beyond the target duration.
- **Recommendation:** Include batch setup and cleanup cost when increasing sample size, and check the next batch cost before reserving its storage.
- **Fix with:** `make-it-fast`
- **Effort:** M (days)

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
