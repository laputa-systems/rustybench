Prefix: RUST
Next: 22

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
