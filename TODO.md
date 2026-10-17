Prefix: RUST
Next: 23

### RUST-22 · P1 · Keep the Clippy gate passing after sample synchronization

- **Area:** Build
- **Where:** `src/benchmark/mod.rs:1068-1102`
- **Problem:** The process-counter boundary closure introduced for synchronized samples triggers Clippy's `unnecessary_lazy_evaluations` and `collapsible_if` lints. `cargo clippy -p rustybench --tests --locked -- --deny warnings` fails, and the configured CI Clippy command denies warnings.
- **Recommendation:** Replace the lazy boolean closure with an eager option and collapse the nested error check without changing synchronization behavior.
- **Fix with:** `none`
- **Effort:** S (hours)

