# Reports and baselines

## Benchmark JSON

Run a benchmark executable with `--format json` to write one schema-1 report
object to stdout:

```sh
cargo bench --bench example -- --format json
```

The report has a `schema` number and a `benchmarks` array. Each record contains
`name`, `median_ns`, `alloc_count`, `alloc_bytes`, `max_alloc_count`,
`max_alloc_bytes`, `sample_count`, and `iter_count`. Time is rounded to integer
nanoseconds. `alloc_count` and `alloc_bytes` summarize per-iteration allocations.

`max_alloc_count` and `max_alloc_bytes` are medians of each sample's maximum
positive net allocation growth since its measured boundary. These peaks are
not divided by the iteration count and are not total live heap usage. Freeing
an input allocated during setup subtracts from the balance: freeing 1 MiB and
then allocating 1 MiB can report `alloc_bytes = 1048576` with
`max_alloc_bytes = 0`. Keep sample size fixed when comparing peak budgets.

Records may include an optional `process_resources` object. Its `status` and
`memory_status` are `supported`, `unsupported`, or `not_applicable`. Supported
Linux records include per-process user and system CPU nanoseconds, voluntary and
involuntary context switches, minor and major page faults, and optional RSS and
PSS byte snapshots from bounded `/proc` reads. macOS reports current RSS from
`task_info`. PSS and process counters remain unsupported there. Missing metrics
remain unavailable, not zero. CPU time, context switches, and page faults are
medians of process-wide sample deltas. RSS and PSS are medians of the ending
snapshots. Both resource snapshots bracket the shared operation boundary.
All workers wait for the ending snapshot before deferred input and output
destruction, so that cleanup is excluded from process-resource measurements.
Older schema-1 reports without `process_resources` remain readable.

## Baseline JSON

The `baseline` tool stores schema-1 JSON with host, platform, architecture, run
mode and counts, wall-time totals, per-run suite times, and aggregated
benchmark metrics. Each benchmark record has `name`, `median_ns`,
`alloc_count`, `alloc_bytes`, `max_alloc_count`, and `max_alloc_bytes`, plus an
optional `run_median_ns` array. Older schema-1 files without that field remain
readable. Process-resource fields from raw benchmark reports are not retained
in baselines. See [collection limitations](commands.md#baseline-tools) before
using a baseline as a budget.

Additive schema-1 fields remain optional. A change that makes a field required
or changes its meaning needs a schema bump and a compatibility test.

The syscall diagnostic has a separate schema-1 JSON report marked
`"diagnostic": true` and `"timing": false`; it is not a benchmark report.
