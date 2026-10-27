Prefix: RUST
Next: 25

### RUST-23 · P1 · JSON max allocation is the sample peak divided by iterations

- **Area:** Data
- **Where:** `src/benchmark/mod.rs:1492`, `src/report.rs:43`, `src/report.rs:69`, `src/alloc.rs:365`
- **Problem:** `max_alloc_count` and `max_alloc_bytes` are documented as maximum live allocations, and `rustybench check` treats them as ceilings. `ThreadAllocInfo::max_count` and `max_size` are the peak live values over a whole sample. `BenchContext::compute_stats` then divides those peaks by `sample_size`, and `BenchmarkRecord::from_stats` rounds the result to an integer. A buffer that allocates 1_048_576 bytes once per iteration, run with `--bench --format json --sample-count 3 --sample-size 100` on nightly-2026-09-15, produced `alloc_bytes` 1048576, `max_alloc_count` 0, and `max_alloc_bytes` 10486. The default tuner raises `sample_size`, so the published peak shrinks as the sample gets longer. The existing synthetic alloc test sets `sample_size` to 1, so it cannot see the division. Proof: ran that harness (blast-radius rung 4).
- **Recommendation:** Stop dividing `max_alloc` by `sample_size` in `compute_stats`. Keep per-iteration scaling on allocation operation tallies. Extend the synthetic report test with `sample_size` greater than 1 and assert the JSON peak stays the configured peak.
- **Fix with:** none
- **Effort:** S

### RUST-24 · P3 · `allocator_api` warns on nightlies after the pin

- **Area:** Build
- **Where:** `src/lib.rs:4`, `README.md:7`, `rust-toolchain.toml:2`
- **Problem:** The crate enables `allocator_api`. README tells users to install the latest nightly. On nightly 1.101.0 (16d1ffac1 2026-10-26) that feature warns as partially stabilized and points at `allocator_ext`. The pinned toolchain `nightly-2026-09-15` still builds cleanly. A warning-free build does not match the README's "latest nightly" instruction, and the feature can disappear.
- **Recommendation:** Move the allocator API onto the stabilized surface, or onto `allocator_ext` for what is still unstable, and say which nightly README actually supports.
- **Fix with:** none
- **Effort:** S
