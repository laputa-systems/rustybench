Prefix: RUST
Next: 35

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
