Prefix: RUST
Next: 35

### RUST-30 · P2 · The syscall diagnostic dispatches its image build as a container command

- **Area:** Operability
- **Where:** `src/bin/rustybench.rs:907`, `src/bin/rustybench.rs:990`, `src/bin/rustybench.rs:1016`
- **Problem:** The initial Docker build goes through `docker_base`, which always
  prefixes `run ... benchmark-syscalls:local`. A recording Docker executable
  confirms that `rustybench syscalls --root ROOT` invokes
  `docker run ... benchmark-syscalls:local build --quiet -t benchmark-syscalls:local .`.
  Docker never receives a build subcommand: a clean host has no image, and an
  existing image would execute a program named `build` inside the container.
  The advertised diagnostic therefore cannot reach collection.
- **Recommendation:** Run the image build as a separate host-side Docker command
  with `current_dir(root)`. Reserve the container-run helper for commands inside
  the image, and check the external argv for both operations.
- **Fix with:** `isolate-effects`
- **Effort:** S (hours)

### RUST-31 · P2 · Syscall collection substitutes group labels for benchmark identities

- **Area:** Data
- **Where:** `src/bin/rustybench.rs:922`, `src/bin/rustybench.rs:940`, `src/rustybench.rs:175`
- **Problem:** The diagnostic parses only unindented branch rows from the human
  tree, then uses those labels as regular-expression filters. With the real
  repository benchmark target, terse listing exposes seven enabled leaves, but
  the current parser selects only `harness_overhead` and `workloads`. A Docker
  fixture returning that actual listing after a successful build response made
  the unmodified tool emit those two group names as benchmark records. Trace
  commands lack `--exact`, collapse multiple functions/arguments, and include a
  wholly ignored group. This defect remains after RUST-30 is repaired; the proof
  covers command construction and selection, not live tracing.
- **Recommendation:** Consume the runner's terse full-path leaf listing, apply
  the same ignore policy, and select each identity with `--exact`. Preserve
  argument/thread identity and reject missing marker bounds. Check nested
  groups, arguments, ignored entries, and regex punctuation at this boundary.
- **Fix with:** `single-source-of-truth`
- **Effort:** S (hours)

### RUST-32 · P2 · Baseline argument forwarding breaks direct benchmark executables

- **Area:** Errors
- **Where:** `src/bin/rustybench.rs:167`, `src/bin/rustybench.rs:353`
- **Problem:** Help accepts `baseline [OPTIONS] -- COMMAND [ARGS...]`, but the
  collector always inserts Cargo's `--` separator. A working prebuilt benchmark
  executable receives `-- --bench --format json ...`, interprets the injected
  flags as filters, and emits no report. Running the same real executable
  directly with `--bench --format json` succeeds; passing it to
  `rustybench baseline --fast --quiet -- EXECUTABLE` exits 2 with
  `could not parse benchmark JSON`. This prevents collection from prebuilt
  executables when Cargo/build overhead should be outside the experiment.
- **Recommendation:** Separate Cargo argument forwarding from direct executable
  forwarding. Preserve the existing Cargo forms and append runner flags directly
  for a harness executable. Verify observable argv and successful collection
  through both command interfaces.
- **Fix with:** `isolate-effects`
- **Effort:** S (hours)

### RUST-33 · P2 · Callable generic bounds are parsed as benchmark parameters

- **Area:** Architecture
- **Where:** `macros/src/lib.rs:334`, `macros/src/lib.rs:346`, `macros/src/lib.rs:126`
- **Problem:** `parse_function` treats the first parenthesized token group after
  the function name as its parameters, including parentheses inside generic
  bounds. Compiling `#[rustybench::bench(types = [fn(u32)])] fn callable<T:
  Fn(u32)>() { rustybench::black_box(std::mem::size_of::<T>()); }` fails with
  E0308: the generated code expects `fn(Bencher)` but receives a zero-argument
  function. The macro mistakes `Fn(u32)` for the benchmark's argument list.
- **Recommendation:** Locate parameters after the complete generic declaration,
  preferably by parsing the signature with the existing `syn` dependency.
  Exercise callable bounds with nonempty argument lists at the public macro
  compilation boundary.
- **Fix with:** `api-surface-and-module-boundaries`
- **Effort:** S (hours)

### RUST-34 · P2 · Generic runtime-argument types escape their scope during macro expansion

- **Area:** Types
- **Where:** `macros/src/lib.rs:106`, `macros/src/lib.rs:119`, `macros/src/lib.rs:142`, `src/lib.rs:550`
- **Problem:** Combining `types` and `args` is supported, but expansion substitutes
  selected generics only into the function call. It copies the argument type
  into an `Arg::<...>` outside the generic function without substitution.
  Compiling `#[rustybench::bench(types = [u8], args = [1u8])] fn echo<T: Copy +
  Send + Sync>(arg: T) -> T { arg }` fails with E0425, `cannot find type T in
  this scope`. The documented combined example uses a nongeneric argument and
  does not exercise this case.
- **Recommendation:** Resolve the argument type for each instantiation, or move
  conversion into a generated generic helper where those names remain in scope.
  Preserve existing borrowing/coercion behavior and cover direct and referenced
  generic argument types.
- **Fix with:** `typing-audit`
- **Effort:** M (days)

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
