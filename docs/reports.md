# Reports and baselines

## Benchmark JSON

Run a benchmark executable with `--format json` to write one schema-1 report
object to stdout:

```sh
cargo bench --bench example -- --format json
```

The report has a `schema` number and a `benchmarks` array. Each record contains
`name`, `median_ns`, `alloc_count`, `alloc_bytes`, `max_alloc_count`,
`max_alloc_bytes`, `sample_count`, and `iter_count`. Time is reported in
nanoseconds. Allocation metrics report counts and bytes, plus maximum live
counts and bytes; values are summarized from the measured samples.

Records may include an optional `process_resources` object. Its `status` and
`memory_status` are `supported`, `unsupported`, or `not_applicable`. Supported
Linux records include per-process user and system CPU nanoseconds, voluntary and
involuntary context switches, minor and major page faults, and optional RSS and
PSS byte snapshots from bounded `/proc` reads. Other platforms report the
Linux-only measurements as unsupported. Missing metrics remain unavailable,
not zero. CPU time, context switches, and page faults are medians of
per-sample deltas. RSS and PSS are medians of the end-of-sample snapshots.
Older schema-1 reports without `process_resources` remain readable.

## Baseline JSON

The `baseline` tool stores schema-1 JSON with host, platform, architecture, run
mode and counts, wall-time totals, per-run suite times, and aggregated
benchmark metrics. Each benchmark record has `name`, `median_ns`,
`alloc_count`, `alloc_bytes`, `max_alloc_count`, and `max_alloc_bytes`, plus an
optional `run_median_ns` array. Older schema-1 files without that field remain
readable. Baseline files are JSON, not tab-separated text.

Additive schema-1 fields remain optional. A change that makes a field required
or changes its meaning needs a schema bump and a compatibility test.

The syscall diagnostic has a separate schema-1 JSON report marked
`"diagnostic": true` and `"timing": false`; it is not a benchmark report.
