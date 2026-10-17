Prefix: RUST
Next: 22

### RUST-21 · P4 · Split `consts` by reparsing instead of by token heuristics

- **Area:** Macros
- **Where:** `macros/src/tokens.rs:33-86`
- **Problem:** The angle-depth heuristic cannot tell a comparison between two identifiers from a generic argument list, so `#[bench(consts = [LEN < 3, 4])]` still swallows the comma after `3` and registers one generic bench instead of two, silently dropping the rest of the list.
- **Recommendation:** Decide the split by parsing: split at commas and keep fragments that parse as the expected type or expression, rather than by tracking angle depth at the token level.
- **Fix with:** `root-cause`
- **Effort:** M (days)
